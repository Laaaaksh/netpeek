import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { SnapshotPayload } from "@/types";

const BANDWIDTH_EVENT = "bandwidth-update";

export function useBandwidth() {
  const [payload, setPayload] = useState<SnapshotPayload | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    invoke<SnapshotPayload | null>("get_latest_snapshot")
      .then((initial) => {
        if (!cancelled && initial) setPayload(initial);
      })
      .catch(() => {
        // No snapshot yet - the first live update will populate the UI.
      });

    listen<SnapshotPayload>(BANDWIDTH_EVENT, (event) => {
      setPayload(event.payload);
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return payload;
}
