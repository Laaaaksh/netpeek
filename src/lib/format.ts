export function formatBps(bps: number): string {
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;

  if (bps >= GB) return `${(bps / GB).toFixed(1)} GB/s`;
  if (bps >= MB) return `${(bps / MB).toFixed(1)} MB/s`;
  if (bps >= KB) return `${(bps / KB).toFixed(0)} KB/s`;
  return `${Math.round(bps)} B/s`;
}
