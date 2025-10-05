import { useEffect, useRef, useState, useCallback } from 'react';
import { VariableSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';

import DisplayConversationEntry from '../NormalizedConversation/DisplayConversationEntry';
import { useEntries } from '@/contexts/EntriesContext';
import {
  AddEntryType,
  PatchTypeWithKey,
  useConversationHistory,
} from '@/hooks/useConversationHistory';
import { Loader2 } from 'lucide-react';
import { TaskAttempt } from 'shared/types';

interface VirtualizedListProps {
  attempt: TaskAttempt;
}

const VirtualizedList = ({ attempt }: VirtualizedListProps) => {
  const [entries, setEntriesState] = useState<PatchTypeWithKey[]>([]);
  const [loading, setLoading] = useState(true);
  const { setEntries, reset } = useEntries();
  const listRef = useRef<List>(null);
  const itemSizeCache = useRef<Map<number, number>>(new Map());
  const rowHeights = useRef<Map<number, number>>(new Map());
  const shouldScrollToBottom = useRef(true);

  useEffect(() => {
    setLoading(true);
    setEntriesState([]);
    reset();
    itemSizeCache.current.clear();
    rowHeights.current.clear();
  }, [attempt.id, reset]);

  const onEntriesUpdated = useCallback(
    (
      newEntries: PatchTypeWithKey[],
      addType: AddEntryType,
      newLoading: boolean
    ) => {
      setEntriesState(newEntries);
      setEntries(newEntries);

      if (loading) {
        setLoading(newLoading);
      }

      // Auto-scroll to bottom when new entries arrive
      if (addType === 'running' && !loading) {
        shouldScrollToBottom.current = true;
        requestAnimationFrame(() => {
          if (listRef.current && shouldScrollToBottom.current) {
            listRef.current.scrollToItem(newEntries.length - 1, 'end');
          }
        });
      } else if (addType === 'initial' && newLoading === false) {
        // Scroll to bottom on initial load
        shouldScrollToBottom.current = true;
        requestAnimationFrame(() => {
          if (listRef.current) {
            listRef.current.scrollToItem(newEntries.length - 1, 'end');
          }
        });
      }
    },
    [loading, setEntries]
  );

  useConversationHistory({ attempt, onEntriesUpdated });

  const getItemSize = useCallback((index: number) => {
    // Return cached size or estimate
    return rowHeights.current.get(index) || 100;
  }, []);

  const setRowHeight = useCallback((index: number, size: number) => {
    if (rowHeights.current.get(index) !== size) {
      rowHeights.current.set(index, size);
      listRef.current?.resetAfterIndex(index);
    }
  }, []);

  const Row = useCallback(
    ({ index, style }: { index: number; style: React.CSSProperties }) => {
      const data = entries[index];
      const rowRef = useRef<HTMLDivElement>(null);

      useEffect(() => {
        if (rowRef.current) {
          const height = rowRef.current.getBoundingClientRect().height;
          setRowHeight(index, height);
        }
      }, [index]);

      if (!data) return null;

      return (
        <div style={style}>
          <div ref={rowRef}>
            {data.type === 'STDOUT' && <p>{data.content}</p>}
            {data.type === 'STDERR' && <p>{data.content}</p>}
            {data.type === 'NORMALIZED_ENTRY' && attempt && (
              <DisplayConversationEntry
                expansionKey={data.patchKey}
                entry={data.content}
                executionProcessId={data.executionProcessId}
                taskAttempt={attempt}
              />
            )}
          </div>
        </div>
      );
    },
    [entries, attempt, setRowHeight]
  );

  return (
    <>
      <div className="flex-1 relative">
        <AutoSizer>
          {({ height, width }) => (
            <>
              <div style={{ height: 8 }} />
              <List
                ref={listRef}
                height={height - 16}
                itemCount={entries.length}
                itemSize={getItemSize}
                width={width}
                itemKey={(index) => `l-${entries[index]?.patchKey || index}`}
                onScroll={({ scrollUpdateWasRequested }) => {
                  // User scrolled manually - disable auto-scroll
                  if (!scrollUpdateWasRequested) {
                    shouldScrollToBottom.current = false;
                  }
                }}
              >
                {Row}
              </List>
              <div style={{ height: 8 }} />
            </>
          )}
        </AutoSizer>
      </div>
      {loading && (
        <div className="absolute top-0 left-0 w-full h-full bg-primary flex flex-col gap-2 justify-center items-center">
          <Loader2 className="h-8 w-8 animate-spin" />
          <p>Loading History</p>
        </div>
      )}
    </>
  );
};

export default VirtualizedList;
