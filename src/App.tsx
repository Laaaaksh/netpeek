import { Header } from "@/components/Header";
import { ProcessTable } from "@/components/ProcessTable";
import { SuggestionsPanel } from "@/components/SuggestionsPanel";
import { useBandwidth } from "@/hooks/useBandwidth";

export default function App() {
  const payload = useBandwidth();
  const processes = payload?.snapshot.processes ?? [];
  const suggestions = payload?.suggestions ?? [];

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
      <Header
        totalDownloadBps={payload?.snapshot.totalDownloadBps ?? 0}
        totalUploadBps={payload?.snapshot.totalUploadBps ?? 0}
        isLive={payload !== null}
      />
      <main className="grid min-h-0 flex-1 grid-cols-[1fr_320px]">
        <section className="min-h-0 overflow-y-auto border-r border-border">
          <ProcessTable processes={processes} />
        </section>
        <aside className="min-h-0 min-w-0 overflow-hidden">
          <SuggestionsPanel suggestions={suggestions} />
        </aside>
      </main>
    </div>
  );
}
