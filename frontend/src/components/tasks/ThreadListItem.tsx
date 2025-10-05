import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { statusBoardColors, statusLabels } from '@/utils/status-labels';
import type { TaskWithAttemptStatus } from 'shared/types';

interface ThreadListItemProps {
  task: TaskWithAttemptStatus;
  onClick: (task: TaskWithAttemptStatus) => void;
}

export function ThreadListItem({ task, onClick }: ThreadListItemProps) {
  const formatRelativeTime = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const getStatusColor = () => {
    const colorVar = statusBoardColors[task.status];
    switch (colorVar) {
      case '--info':
        return 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300';
      case '--warning':
        return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300';
      case '--success':
        return 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300';
      case '--destructive':
        return 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300';
      default:
        return 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300';
    }
  };

  return (
    <Card
      className="p-4 cursor-pointer hover:bg-accent/50 transition-colors border rounded-lg active:scale-[0.98]"
      onClick={() => onClick(task)}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <h3 className="font-medium text-sm truncate">{task.title}</h3>
          <p className="text-xs text-muted-foreground mt-1">
            {formatRelativeTime(task.updated_at)}
          </p>
        </div>
        <Badge className={getStatusColor()}>
          {statusLabels[task.status]}
        </Badge>
      </div>
    </Card>
  );
}
