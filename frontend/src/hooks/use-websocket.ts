"use client";

import { useCallback, useEffect, useRef, useState } from "react";

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001/ws";

// ============================================================================
// WebSocket Message Types (matching Rust types)
// ============================================================================

export type Platform = "kalshi" | "polymarket";

export interface SubscriptionType {
  type: "price" | "order_book" | "trades";
  platform: Platform;
  market_id: string;
}

export interface ClientMessage {
  type: "subscribe" | "unsubscribe" | "ping";
  subscription?: SubscriptionType;
  timestamp?: number;
}

export interface PriceUpdate {
  type: "price_update";
  platform: Platform;
  market_id: string;
  yes_price: string;
  no_price: string;
  timestamp: string;
}

export interface OrderBookLevel {
  price: string;
  quantity: string;
  order_count: number | null;
}

export interface OrderBookUpdate {
  type: "order_book_update";
  platform: Platform;
  market_id: string;
  update_type: "snapshot" | "delta";
  yes_bids: OrderBookLevel[];
  yes_asks: OrderBookLevel[];
  no_bids: OrderBookLevel[];
  no_asks: OrderBookLevel[];
  timestamp: string;
}

export interface Trade {
  id: string;
  market_id: string;
  platform: Platform;
  timestamp: string;
  price: string;
  quantity: string;
  outcome: "Yes" | "No";
  side: "Buy" | "Sell" | null;
}

export interface TradeUpdate {
  type: "trade_update";
  platform: Platform;
  market_id: string;
  trade: Trade;
}

export interface SubscribedMessage {
  type: "subscribed";
  subscription: SubscriptionType;
}

export interface UnsubscribedMessage {
  type: "unsubscribed";
  subscription: SubscriptionType;
}

export interface ErrorMessage {
  type: "error";
  code: string;
  message: string;
}

export interface PongMessage {
  type: "pong";
  client_timestamp: number;
  server_timestamp: number;
}

export interface ConnectionStatusMessage {
  type: "connection_status";
  platform: Platform;
  status: "connected" | "connecting" | "disconnected" | "failed";
}

export type ServerMessage =
  | PriceUpdate
  | OrderBookUpdate
  | TradeUpdate
  | SubscribedMessage
  | UnsubscribedMessage
  | ErrorMessage
  | PongMessage
  | ConnectionStatusMessage;

// ============================================================================
// Connection State
// ============================================================================

export type ConnectionState =
  | "connecting"
  | "connected"
  | "disconnected"
  | "reconnecting";

// ============================================================================
// WebSocket Hook
// ============================================================================

interface UseWebSocketOptions {
  /** Auto-connect on mount */
  autoConnect?: boolean;
  /** Reconnect automatically on disconnect */
  autoReconnect?: boolean;
  /** Maximum reconnection attempts */
  maxReconnectAttempts?: number;
  /** Base delay for reconnection (ms) */
  reconnectDelay?: number;
  /** Ping interval (ms) */
  pingInterval?: number;
}

interface UseWebSocketReturn {
  /** Current connection state */
  connectionState: ConnectionState;
  /** Last error message */
  error: string | null;
  /** Connect to WebSocket */
  connect: () => void;
  /** Disconnect from WebSocket */
  disconnect: () => void;
  /** Subscribe to a market channel */
  subscribe: (subscription: SubscriptionType) => void;
  /** Unsubscribe from a market channel */
  unsubscribe: (subscription: SubscriptionType) => void;
  /** Register a message handler */
  onMessage: (handler: (message: ServerMessage) => void) => () => void;
  /** Current latency (ms) */
  latency: number | null;
}

export const useWebSocket = (
  options: UseWebSocketOptions = {},
): UseWebSocketReturn => {
  const {
    autoConnect = true,
    autoReconnect = true,
    maxReconnectAttempts = 5,
    reconnectDelay = 1000,
    pingInterval = 30000,
  } = options;

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const messageHandlersRef = useRef<Set<(message: ServerMessage) => void>>(
    new Set(),
  );

  const [connectionState, setConnectionState] =
    useState<ConnectionState>("disconnected");
  const [error, setError] = useState<string | null>(null);
  const [latency, setLatency] = useState<number | null>(null);

  // Clear timeouts
  const clearTimeouts = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
      pingIntervalRef.current = null;
    }
  }, []);

  // Send a message
  const sendMessage = useCallback((message: ClientMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    }
  }, []);

  // Start ping interval
  const startPingInterval = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
    }
    pingIntervalRef.current = setInterval(() => {
      sendMessage({ type: "ping", timestamp: Date.now() });
    }, pingInterval);
  }, [pingInterval, sendMessage]);

  // Connect to WebSocket
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    clearTimeouts();
    setConnectionState("connecting");
    setError(null);

    try {
      const ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        setConnectionState("connected");
        reconnectAttemptsRef.current = 0;
        startPingInterval();
      };

      ws.onclose = (event) => {
        clearTimeouts();
        setConnectionState("disconnected");

        // Attempt to reconnect if enabled and not intentional close
        if (autoReconnect && event.code !== 1000) {
          if (reconnectAttemptsRef.current < maxReconnectAttempts) {
            const delay =
              reconnectDelay * Math.pow(2, reconnectAttemptsRef.current);
            setConnectionState("reconnecting");
            reconnectTimeoutRef.current = setTimeout(() => {
              reconnectAttemptsRef.current++;
              connect();
            }, delay);
          } else {
            setError("Max reconnection attempts reached");
          }
        }
      };

      ws.onerror = () => {
        setError("WebSocket error occurred");
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data) as ServerMessage;

          // Handle pong for latency measurement
          if (message.type === "pong") {
            const clientTimestamp = (message as PongMessage).client_timestamp;
            setLatency(Date.now() - clientTimestamp);
          }

          // Notify all handlers
          messageHandlersRef.current.forEach((handler) => handler(message));
        } catch (e) {
          console.error("Failed to parse WebSocket message:", e);
        }
      };

      wsRef.current = ws;
    } catch (e) {
      setError(`Failed to connect: ${e}`);
      setConnectionState("disconnected");
    }
  }, [
    autoReconnect,
    clearTimeouts,
    maxReconnectAttempts,
    reconnectDelay,
    startPingInterval,
  ]);

  // Disconnect from WebSocket
  const disconnect = useCallback(() => {
    clearTimeouts();
    if (wsRef.current) {
      wsRef.current.close(1000, "Client disconnect");
      wsRef.current = null;
    }
    setConnectionState("disconnected");
  }, [clearTimeouts]);

  // Subscribe to a market channel
  const subscribe = useCallback(
    (subscription: SubscriptionType) => {
      sendMessage({ type: "subscribe", subscription });
    },
    [sendMessage],
  );

  // Unsubscribe from a market channel
  const unsubscribe = useCallback(
    (subscription: SubscriptionType) => {
      sendMessage({ type: "unsubscribe", subscription });
    },
    [sendMessage],
  );

  // Register a message handler
  const onMessage = useCallback((handler: (message: ServerMessage) => void) => {
    messageHandlersRef.current.add(handler);
    return () => {
      messageHandlersRef.current.delete(handler);
    };
  }, []);

  // Auto-connect on mount
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    connectionState,
    error,
    connect,
    disconnect,
    subscribe,
    unsubscribe,
    onMessage,
    latency,
  };
};
