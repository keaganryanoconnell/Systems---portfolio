"use client";

import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: { componentStack: string }) {
    console.error("[ErrorBoundary] Uncaught error:", error, info.componentStack);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div className="flex items-center justify-center min-h-[200px]">
          <div className="cyber-panel rounded p-6 max-w-lg">
            <div className="flex items-center gap-3 mb-4">
              <span className="text-2xl text-red">✕</span>
              <h2 className="text-sm font-mono font-bold text-red">
                RUNTIME_EXCEPTION
              </h2>
            </div>
            <p className="text-xs font-mono text-text-soft mb-3">
              A critical error occurred in this view.
            </p>
            <div className="bg-bg border border-red/20 rounded p-3 mb-4 overflow-auto max-h-[120px]">
              <code className="text-[10px] font-mono text-red">
                {this.state.error?.message || "Unknown error"}
              </code>
            </div>
            <button
              onClick={this.handleReset}
              className="bg-blue-bg border border-blue-border text-blue text-[10px] font-mono font-bold px-4 py-2 rounded hover:bg-blue/10 transition-all"
            >
              ⟳ RETRY
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
