import {
  AlertTriangle,
  ArrowUpRight,
  Cloud,
  Info,
  RefreshCw,
  TimerReset,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import type { Suggestion, SuggestionKind } from "@/types";
import type { ComponentType } from "react";

const ICON: Record<SuggestionKind, ComponentType<{ className?: string }>> = {
  "top-consumer": ArrowUpRight,
  sync: Cloud,
  update: RefreshCw,
  backup: TimerReset,
  background: AlertTriangle,
  info: Info,
};

interface SuggestionsPanelProps {
  suggestions: Suggestion[];
}

export function SuggestionsPanel({ suggestions }: SuggestionsPanelProps) {
  if (suggestions.length === 0) {
    return (
      <div className="flex h-full items-center justify-center px-4 text-center text-sm text-muted-foreground">
        No notable activity yet. Suggestions show up here once something
        stands out.
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2.5 overflow-y-auto p-3">
      {suggestions.map((s) => {
        const Icon = ICON[s.kind];
        return (
          <Card key={s.id} size="sm">
            <CardContent className="flex gap-3">
              <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
              <div className="flex flex-col gap-0.5 text-left">
                <div className="text-sm font-medium leading-snug">
                  {s.title}
                </div>
                <div className="text-xs leading-relaxed text-muted-foreground">
                  {s.detail}
                </div>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
