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
  background: "Background",
};

const CATEGORY_VARIANT: Record<Category, "default" | "secondary" | "outline"> = {
  app: "default",
  sync: "secondary",
  update: "secondary",
  backup: "secondary",
  background: "outline",
};

interface ProcessTableProps {
  processes: ProcessRate[];
}

export function ProcessTable({ processes }: ProcessTableProps) {
  if (processes.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Waiting for the first bandwidth sample…
      </div>
    );
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
        {processes.map((p) => (
          <TableRow key={p.groupKey}>
            <TableCell className="font-medium">
              {p.displayName}
              {p.pidCount > 1 && (
                <span className="ml-1.5 text-xs text-muted-foreground">
                  ({p.pidCount} processes)
                </span>
              )}
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
        ))}
      </TableBody>
    </Table>
  );
}
