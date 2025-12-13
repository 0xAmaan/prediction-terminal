"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { Send, Loader2, User, Bot, AlertCircle, Search, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import ReactMarkdown from "react-markdown";
import { api } from "@/lib/api";
import type { ChatMessage } from "@/lib/types";

interface ResearchChatProps {
  platform: string;
  marketId: string;
  isFollowUpInProgress?: boolean;
  disabled?: boolean;
  onResearchTriggered?: () => void;
}

export function ResearchChat({
  platform,
  marketId,
  isFollowUpInProgress = false,
  disabled = false,
  onResearchTriggered,
}: ResearchChatProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Auto-scroll to bottom when messages change
  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // Load chat history
  const loadChat = useCallback(async () => {
    setIsLoading(true);
    setLoadError(null);
    try {
      const history = await api.getChatHistory(platform, marketId);
      setMessages(history.messages);
    } catch (e) {
      console.error("Failed to load chat history:", e);
      setLoadError("Failed to load chat history. Please try again.");
    } finally {
      setIsLoading(false);
    }
  }, [platform, marketId]);

  // Load chat history on mount
  useEffect(() => {
    loadChat();
  }, [loadChat]);

  // Handle sending a message
  const handleSend = async () => {
    const trimmedInput = input.trim();
    if (!trimmedInput || isSending) return;

    setIsSending(true);
    setError(null);

    // Optimistically add user message
    const optimisticUserMessage: ChatMessage = {
      id: `temp-${Date.now()}`,
      role: "user",
      content: trimmedInput,
      created_at: new Date().toISOString(),
      research_triggered: false,
    };
    setMessages((prev) => [...prev, optimisticUserMessage]);
    setInput("");

    try {
      const assistantMessage = await api.sendChatMessage(
        platform,
        marketId,
        trimmedInput,
      );
      // Replace optimistic message with server response and add assistant message
      setMessages((prev) => {
        // Remove the optimistic message and fetch fresh from the response
        // The server saved both messages, so we reload to get proper IDs
        return [...prev.slice(0, -1), { ...optimisticUserMessage, id: `user-${Date.now()}` }, assistantMessage];
      });

      // Notify parent if research was triggered so it can refresh the report
      if (assistantMessage.research_triggered && onResearchTriggered) {
        onResearchTriggered();
      }
    } catch (e) {
      console.error("Failed to send message:", e);
      setError("Failed to send message. Please try again.");
      // Remove optimistic message on error
      setMessages((prev) => prev.slice(0, -1));
      setInput(trimmedInput); // Restore input
    } finally {
      setIsSending(false);
      inputRef.current?.focus();
    }
  };

  // Handle keyboard shortcuts
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
    if (e.key === "Escape") {
      setInput("");
      inputRef.current?.blur();
    }
  };

  // Format timestamp
  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  return (
    <div className="flex flex-col h-full min-h-0">
      {/* Chat header */}
      <div className="flex-shrink-0 px-4 py-3 border-b border-border/30">
        <h3 className="font-medium text-sm">Follow-up Questions</h3>
        <p className="text-xs text-muted-foreground">
          Ask questions about the research
        </p>
      </div>

      {/* Messages area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 min-h-0">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : loadError ? (
          <div className="text-center py-8">
            <AlertCircle className="h-8 w-8 mx-auto mb-2 text-red-500" />
            <p className="text-sm text-red-400 mb-3">{loadError}</p>
            <Button
              variant="outline"
              size="sm"
              className="gap-2"
              onClick={loadChat}
            >
              <RefreshCw className="h-4 w-4" />
              Retry
            </Button>
          </div>
        ) : messages.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <Bot className="h-8 w-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No messages yet</p>
            <p className="text-xs mt-1">
              Ask a question about the research above
            </p>
          </div>
        ) : (
          messages.map((message) => (
            <div
              key={message.id}
              className={cn(
                "flex gap-3",
                message.role === "user" ? "flex-row-reverse" : "flex-row",
              )}
            >
              {/* Avatar */}
              <div
                className={cn(
                  "flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center",
                  message.role === "user"
                    ? "bg-primary/20 text-primary"
                    : "bg-muted text-muted-foreground",
                )}
              >
                {message.role === "user" ? (
                  <User className="h-4 w-4" />
                ) : (
                  <Bot className="h-4 w-4" />
                )}
              </div>

              {/* Message bubble */}
              <div
                className={cn(
                  "max-w-[80%] rounded-lg px-3 py-2",
                  message.role === "user"
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted",
                )}
              >
                {message.role === "user" ? (
                  <p className="text-sm whitespace-pre-wrap">{message.content}</p>
                ) : (
                  <div className="text-sm prose prose-sm prose-invert max-w-none [&>p]:mb-2 [&>p:last-child]:mb-0 [&>ul]:mb-2 [&>ul]:list-disc [&>ul]:pl-4 [&>ol]:mb-2 [&>ol]:list-decimal [&>ol]:pl-4">
                    <ReactMarkdown>{message.content}</ReactMarkdown>
                  </div>
                )}
                <div
                  className={cn(
                    "text-xs mt-1 flex items-center gap-2",
                    message.role === "user"
                      ? "text-primary-foreground/70"
                      : "text-muted-foreground",
                  )}
                >
                  <span>{formatTime(message.created_at)}</span>
                  {message.research_triggered && (
                    <span className="text-yellow-400">Research triggered</span>
                  )}
                </div>
              </div>
            </div>
          ))
        )}

        {/* Loading indicator while sending */}
        {isSending && (
          <div className="flex gap-3">
            <div className="w-8 h-8 rounded-full bg-muted flex items-center justify-center">
              <Bot className="h-4 w-4 text-muted-foreground" />
            </div>
            <div className="bg-muted rounded-lg px-3 py-2 flex items-center gap-2">
              <Loader2 className="h-4 w-4 animate-spin" />
              <span className="text-sm text-muted-foreground">
                {isFollowUpInProgress ? "Researching..." : "Thinking..."}
              </span>
            </div>
          </div>
        )}

        {/* Follow-up research in progress indicator (from WebSocket) */}
        {isFollowUpInProgress && !isSending && (
          <div className="flex gap-3">
            <div className="w-8 h-8 rounded-full bg-primary/20 flex items-center justify-center">
              <Search className="h-4 w-4 text-primary" />
            </div>
            <div className="bg-primary/10 border border-primary/30 rounded-lg px-3 py-2 flex items-center gap-2">
              <Loader2 className="h-4 w-4 animate-spin text-primary" />
              <span className="text-sm text-primary">
                Conducting follow-up research and updating document...
              </span>
            </div>
          </div>
        )}

        {/* Error message */}
        {error && (
          <div className="flex items-center gap-2 text-red-400 text-sm p-2 bg-red-500/10 rounded-lg">
            <AlertCircle className="h-4 w-4" />
            <span>{error}</span>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div className="flex-shrink-0 p-4 border-t border-border/30">
        {disabled ? (
          <div className="text-center py-2 text-muted-foreground text-sm">
            Chat disabled while viewing historical version
          </div>
        ) : (
          <>
            <div className="flex gap-2">
              <textarea
                ref={inputRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={isFollowUpInProgress ? "Research in progress..." : "Ask a follow-up question..."}
                className="flex-1 min-h-[40px] max-h-[120px] px-3 py-2 text-sm bg-muted border border-border/50 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-primary/50 disabled:opacity-50"
                disabled={isSending || isFollowUpInProgress}
                rows={1}
              />
              <Button
                size="icon"
                onClick={handleSend}
                disabled={!input.trim() || isSending || isFollowUpInProgress}
                className="flex-shrink-0"
              >
                {isSending || isFollowUpInProgress ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Send className="h-4 w-4" />
                )}
              </Button>
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              Press Enter to send, Shift+Enter for new line
            </p>
          </>
        )}
      </div>
    </div>
  );
}
