import { useEffect, useRef, useState, useCallback } from 'react';
import { FixedSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { AlertCircle } from 'lucide-react';
import { useLogStream } from '@/hooks/useLogStream';
import RawLogText from '@/components/common/RawLogText';

interface ProcessLogsViewerProps {
  processId: string;
}

const ITEM_HEIGHT = 24; // Approximate height for log lines

export default function ProcessLogsViewer({
  processId,
}: ProcessLogsViewerProps) {
  const listRef = useRef<List>(null);
  const didInitScroll = useRef(false);
  const prevLenRef = useRef(0);
  const [atBottom, setAtBottom] = useState(true);

  const { logs, error } = useLogStream(processId);

  // 1) Initial jump to bottom once data appears
  useEffect(() => {
    if (!didInitScroll.current && logs.length > 0) {
      didInitScroll.current = true;
      requestAnimationFrame(() => {
        if (listRef.current) {
          listRef.current.scrollToItem(logs.length - 1, 'end');
        }
      });
    }
  }, [logs.length]);

  // 2) If there's a large append and we're at bottom, force-stick to the last item
  useEffect(() => {
    const prev = prevLenRef.current;
    const grewBy = logs.length - prev;
    prevLenRef.current = logs.length;

    const LARGE_BURST = 10;
    if (grewBy >= LARGE_BURST && atBottom && logs.length > 0) {
      requestAnimationFrame(() => {
        if (listRef.current) {
          listRef.current.scrollToItem(logs.length - 1, 'end');
        }
      });
    }
  }, [logs.length, atBottom]);

  const handleScroll = useCallback(
    ({ scrollOffset, scrollUpdateWasRequested }: any) => {
      // Only update atBottom state if this was a user-initiated scroll
      if (!scrollUpdateWasRequested && listRef.current) {
        // Check if we're near the bottom (within 50px)
        const isNearBottom =
          scrollOffset + (listRef.current as any)._outerRef?.clientHeight >=
          logs.length * ITEM_HEIGHT - 50;
        setAtBottom(isNearBottom);
      }
    },
    [logs.length]
  );

  const Row = useCallback(
    ({ index, style }: { index: number; style: React.CSSProperties }) => {
      const entry = logs[index];
      return (
        <div style={style}>
          <RawLogText
            content={entry.content}
            channel={entry.type === 'STDERR' ? 'stderr' : 'stdout'}
            className="text-sm px-4 py-1"
          />
        </div>
      );
    },
    [logs]
  );

  return (
    <div className="h-full">
      {logs.length === 0 && !error ? (
        <div className="p-4 text-center text-muted-foreground text-sm">
          No logs available
        </div>
      ) : error ? (
        <div className="p-4 text-center text-destructive text-sm">
          <AlertCircle className="h-4 w-4 inline mr-2" />
          {error}
        </div>
      ) : (
        <AutoSizer>
          {({ height, width }) => (
            <List
              ref={listRef}
              height={height}
              itemCount={logs.length}
              itemSize={ITEM_HEIGHT}
              width={width}
              onScroll={handleScroll}
              className="flex-1 rounded-lg"
            >
              {Row}
            </List>
          )}
        </AutoSizer>
      )}
    </div>
  );
}
