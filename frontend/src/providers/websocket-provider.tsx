"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type {
  ConnectionState,
  ServerMessage,
  SubscriptionType,
} from "@/hooks/use-websocket";

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001/ws";

// ============================================================================
// WebSocket Context Types
// ============================================================================

interface WebSocketContextValue {
  /** Current connection state */
  connectionState: ConnectionState;
  /** Last error message */
  error: string | null;
  /** Current latency (ms) */
  latency: number | null;
  /** Subscribe to a market channel */
  subscribe: (subscription: SubscriptionType) => void;
  /** Unsubscribe from a market channel */
  unsubscribe: (subscription: SubscriptionType) => void;
  /** Register a message handler - returns cleanup function */
  onMessage: (handler: (message: ServerMessage) => void) => () => void;
}

const WebSocketContext = createContext<WebSocketContextValue | null>(null);

// ============================================================================
// WebSocket Provider
// ============================================================================

interface WebSocketProviderProps {
  children: ReactNode;
  /** Auto-connect on mount (default: true) */
  autoConnect?: boolean;
  /** Reconnect automatically on disconnect (default: true) */
  autoReconnect?: boolean;
  /** Maximum reconnection attempts (default: 5) */
  maxReconnectAttempts?: number;
  /** Base delay for reconnection in ms (default: 1000) */
  reconnectDelay?: number;
  /** Ping interval in ms (default: 30000) */
  pingInterval?: number;
}

export const WebSocketProvider = ({
  children,
  autoConnect = true,
  autoReconnect = true,
  maxReconnectAttempts = 5,
  reconnectDelay = 1000,
  pingInterval = 30000,
}: WebSocketProviderProps) => {
  // Connection state
  const [connectionState, setConnectionState] =
    useState<ConnectionState>("disconnected");
  const [error, setError] = useState<string | null>(null);
  const [latency, setLatency] = useState<number | null>(null);

  // Refs for WebSocket and timers
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Message handlers set - persists across re-renders
  const messageHandlersRef = useRef<Set<(message: ServerMessage) => void>>(
    new Set()
  );

  // Track active subscriptions for re-subscribing on reconnect
  const activeSubscriptionsRef = useRef<Map<string, SubscriptionType>>(
    new Map()
  );

  // Generate a unique key for a subscription
  const getSubscriptionKey = useCallback((sub: SubscriptionType): string => {
    if (sub.type === "global_news") {
      return "global_news";
    }
    return `${sub.type}:${sub.platform}:${sub.market_id}`;
  }, []);

  // Clear all timers
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

  // Send a message to the WebSocket
  const sendMessage = useCallback(
    (message: { type: string; subscription?: SubscriptionType; timestamp?: number }) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify(message));
      }
    },
    []
  );

  // Start ping interval for keep-alive
  const startPingInterval = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
    }
    pingIntervalRef.current = setInterval(() => {
      sendMessage({ type: "ping", timestamp: Date.now() });
    }, pingInterval);
  }, [pingInterval, sendMessage]);

  // Re-subscribe to all active subscriptions (after reconnect)
  const resubscribeAll = useCallback(() => {
    activeSubscriptionsRef.current.forEach((subscription) => {
      sendMessage({ type: "subscribe", subscription });
    });
  }, [sendMessage]);

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
        console.log("[WebSocket] Connected to", WS_URL);
        setConnectionState("connected");
        reconnectAttemptsRef.current = 0;
        startPingInterval();
        // Re-subscribe to any active subscriptions after reconnect
        resubscribeAll();
      };

      ws.onclose = (event) => {
        console.log("[WebSocket] Disconnected, code:", event.code);
        clearTimeouts();
        setConnectionState("disconnected");

        // Attempt to reconnect if enabled and not intentional close
        if (autoReconnect && event.code !== 1000) {
          if (reconnectAttemptsRef.current < maxReconnectAttempts) {
            const delay =
              reconnectDelay * Math.pow(2, reconnectAttemptsRef.current);
            console.log(
              `[WebSocket] Reconnecting in ${delay}ms (attempt ${reconnectAttemptsRef.current + 1}/${maxReconnectAttempts})`
            );
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
          if (message.type === "pong" && "client_timestamp" in message) {
            setLatency(Date.now() - message.client_timestamp);
          }

          // Notify all registered handlers
          messageHandlersRef.current.forEach((handler) => {
            try {
              handler(message);
            } catch (e) {
              console.error("[WebSocket] Handler error:", e);
            }
          });
        } catch (e) {
          console.error("[WebSocket] Failed to parse message:", e);
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
    resubscribeAll,
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
      const key = getSubscriptionKey(subscription);
      activeSubscriptionsRef.current.set(key, subscription);
      sendMessage({ type: "subscribe", subscription });
    },
    [getSubscriptionKey, sendMessage]
  );

  // Unsubscribe from a market channel
  const unsubscribe = useCallback(
    (subscription: SubscriptionType) => {
      const key = getSubscriptionKey(subscription);
      activeSubscriptionsRef.current.delete(key);
      sendMessage({ type: "unsubscribe", subscription });
    },
    [getSubscriptionKey, sendMessage]
  );

  // Register a message handler - returns cleanup function
  const onMessage = useCallback(
    (handler: (message: ServerMessage) => void): (() => void) => {
      messageHandlersRef.current.add(handler);
      return () => {
        messageHandlersRef.current.delete(handler);
      };
    },
    []
  );

  // Auto-connect on mount
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return () => {
      disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run on mount/unmount

  const value: WebSocketContextValue = {
    connectionState,
    error,
    latency,
    subscribe,
    unsubscribe,
    onMessage,
  };

  return (
    <WebSocketContext.Provider value={value}>
      {children}
    </WebSocketContext.Provider>
  );
};

// ============================================================================
// Hook to use WebSocket context
// ============================================================================

export const useWebSocketContext = (): WebSocketContextValue => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error(
      "useWebSocketContext must be used within a WebSocketProvider"
    );
  }
  return context;
};
