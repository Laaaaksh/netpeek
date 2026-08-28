import { Fragment, useState } from "react";
import { ChevronDown, ChevronRight, Puzzle } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { formatBps } from "@/lib/format";
import type { Category, ProcessRate } from "@/types";

const CATEGORY_LABEL: Record<Category, string> = {
  app: "App",
  sync: "File sync",
  update: "Update",
  backup: "Backup",
  system: "System",
  unrecognized: "Unrecognized",
};

const CATEGORY_VARIANT: Record<Category, "default" | "secondary" | "outline"> = {
  app: "default",
  sync: "secondary",
  update: "secondary",
  backup: "secondary",
  system: "outline",
  unrecognized: "outline",
};

interface ProcessTableProps {
  processes: ProcessRate[];
}

export function ProcessTable({ processes }: ProcessTableProps) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  if (processes.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Waiting for the first bandwidth sample…
      </div>
    );
  }

  function toggle(groupKey: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(groupKey)) {
        next.delete(groupKey);
      } else {
        next.add(groupKey);
      }
      return next;
    });
  }

  return (
    <Table>
      <TableHeader className="sticky top-0 z-10 bg-background">
        <TableRow>
          <TableHead>Process</TableHead>
          <TableHead>Category</TableHead>
          <TableHead className="text-right">Download</TableHead>
          <TableHead className="text-right">Upload</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {processes.map((p) => {
          const isExpanded = expanded.has(p.groupKey);
          return (
            <Fragment key={p.groupKey}>
              <TableRow
                aria-expanded={isExpanded}
                className="cursor-pointer"
                onClick={() => toggle(p.groupKey)}
              >
                <TableCell className="font-medium">
                  <div className="flex items-center gap-1">
                    {isExpanded ? (
                      <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" />
                    ) : (
                      <ChevronRight className="size-3.5 shrink-0 text-muted-foreground" />
                    )}
                    {p.displayName}
                    {p.pidCount > 1 && (
                      <span className="text-xs text-muted-foreground">
                        ({p.pidCount} processes)
                      </span>
                    )}
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant={CATEGORY_VARIANT[p.category]}>
                    {CATEGORY_LABEL[p.category]}
                  </Badge>
                </TableCell>
                <TableCell className="text-right font-mono tabular-nums">
                  {formatBps(p.downloadBps)}
                </TableCell>
                <TableCell className="text-right font-mono tabular-nums">
                  {formatBps(p.uploadBps)}
                </TableCell>
              </TableRow>
              {isExpanded && (
                <TableRow className="hover:bg-transparent">
                  <TableCell colSpan={4} className="whitespace-normal bg-muted/30 py-3">
                    <ProcessDetail process={p} />
                  </TableCell>
                </TableRow>
              )}
            </Fragment>
          );
        })}
      </TableBody>
    </Table>
  );
}

function ProcessDetail({ process }: { process: ProcessRate }) {
  return (
    <div className="flex flex-col gap-3 text-sm">
      <div>
        <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
          What is this?
        </div>
        <p className="mt-0.5">{process.whatItIs}</p>
      </div>
      <div>
        <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
          Should I do anything?
        </div>
        <p className="mt-0.5">{process.verdict}</p>
      </div>
      {process.breakdown && process.breakdown.length > 0 && (
        <div>
          <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            What's using the data
          </div>
          <ul className="mt-1 flex flex-col gap-1">
            {process.breakdown.map((entry) => (
              <li
                key={entry.label}
                className="flex items-center justify-between gap-3 rounded-md bg-background px-2 py-1 ring-1 ring-border"
              >
                <span className="flex items-center gap-1.5">
                  {entry.isExtension && (
                    <Puzzle className="size-3.5 shrink-0 text-muted-foreground" />
                  )}
                  {entry.label}
                  {entry.pidCount > 1 && (
                    <span className="text-xs text-muted-foreground">
                      ({entry.pidCount})
                    </span>
                  )}
                </span>
                <span className="font-mono text-xs tabular-nums text-muted-foreground">
                  {formatBps(entry.downloadBps + entry.uploadBps)}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {process.taskManagerHint && (
        <p className="rounded-md bg-background px-2 py-1.5 text-xs text-muted-foreground ring-1 ring-border">
          {process.taskManagerHint}
        </p>
      )}
    </div>
  );
}
