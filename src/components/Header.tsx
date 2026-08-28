import { ArrowDown, ArrowUp, Moon, Sun } from "lucide-react";
import { formatBps } from "@/lib/format";
import { useTheme } from "@/hooks/useTheme";

interface HeaderProps {
  totalDownloadBps: number;
  totalUploadBps: number;
  isLive: boolean;
}

export function Header({ totalDownloadBps, totalUploadBps, isLive }: HeaderProps) {
  const { theme, toggle } = useTheme();

  return (
    <header className="flex shrink-0 items-center justify-between border-b border-border px-6 py-4">
      <div className="flex items-center gap-3">
        <div className="flex size-9 items-center justify-center rounded-lg bg-primary text-primary-foreground font-heading text-base font-semibold">
          N
        </div>
        <div>
          <h1 className="font-heading text-lg font-semibold leading-tight">
            Netpeek
          </h1>
          <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <span
              className={`size-1.5 rounded-full ${isLive ? "bg-emerald-500" : "bg-muted-foreground/40"}`}
            />
            {isLive ? "Live" : "Waiting for first sample…"}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-6">
        <div className="flex items-center gap-2 rounded-lg border border-border px-3 py-2">
          <ArrowDown className="size-4 text-blue-500" />
          <div className="text-left">
            <div className="text-[10px] uppercase tracking-wide text-muted-foreground">
              Download
            </div>
            <div className="font-mono text-sm font-semibold tabular-nums">
              {formatBps(totalDownloadBps)}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2 rounded-lg border border-border px-3 py-2">
          <ArrowUp className="size-4 text-orange-500" />
          <div className="text-left">
            <div className="text-[10px] uppercase tracking-wide text-muted-foreground">
              Upload
            </div>
            <div className="font-mono text-sm font-semibold tabular-nums">
              {formatBps(totalUploadBps)}
            </div>
          </div>
        </div>

        <button
          type="button"
          onClick={toggle}
          aria-label="Toggle color theme"
          className="flex size-9 items-center justify-center rounded-lg border border-border text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        >
          {theme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
        </button>
      </div>
    </header>
  );
}
