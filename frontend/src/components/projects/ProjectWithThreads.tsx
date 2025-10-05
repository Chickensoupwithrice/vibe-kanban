import { useState } from 'react';
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card.tsx';
import { Badge } from '@/components/ui/badge.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu.tsx';
import { Button } from '@/components/ui/button.tsx';
import {
  Calendar,
  Edit,
  ExternalLink,
  FolderOpen,
  MoreHorizontal,
  Trash2,
  ChevronDown,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { Project, TaskWithAttemptStatus } from 'shared/types';
import { useOpenProjectInEditor } from '@/hooks/useOpenProjectInEditor';
import { projectsApi } from '@/lib/api';
import { ThreadListItem } from '@/components/tasks/ThreadListItem';

type Props = {
  project: Project;
  tasks: TaskWithAttemptStatus[];
  onThreadClick: (projectId: string, taskId: string) => void;
  fetchProjects?: () => void;
  setError?: (error: string) => void;
  onEdit?: (project: Project) => void;
};

function ProjectWithThreads({
  project,
  tasks,
  onThreadClick,
  fetchProjects,
  setError,
  onEdit,
}: Props) {
  const navigate = useNavigate();
  const handleOpenInEditor = useOpenProjectInEditor(project);
  const [showAll, setShowAll] = useState(false);

  const sortedTasks = [...tasks].sort((a, b) => {
    const dateA = new Date(a.updated_at || a.created_at).getTime();
    const dateB = new Date(b.updated_at || b.created_at).getTime();
    return dateB - dateA;
  });

  const displayedTasks = showAll ? sortedTasks : sortedTasks.slice(0, 5);
  const hasMore = sortedTasks.length > 5;

  const handleDelete = async (id: string, name: string) => {
    if (
      !confirm(
        `Are you sure you want to delete "${name}"? This action cannot be undone.`
      )
    )
      return;

    try {
      await projectsApi.delete(id);
      fetchProjects?.();
    } catch (error) {
      console.error('Failed to delete project:', error);
      setError?.('Failed to delete project');
    }
  };

  const handleEdit = (project: Project) => {
    onEdit?.(project);
  };

  const handleOpenInIDE = () => {
    handleOpenInEditor();
  };

  return (
    <Card className="border">
      <CardHeader className="pb-4">
        <div className="flex items-start justify-between">
          <CardTitle className="text-lg">{project.name}</CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="secondary">Active</Badge>
            <DropdownMenu>
              <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
                <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation();
                    navigate(`/projects/${project.id}`);
                  }}
                >
                  <ExternalLink className="mr-2 h-4 w-4" />
                  View Project
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation();
                    handleOpenInIDE();
                  }}
                >
                  <FolderOpen className="mr-2 h-4 w-4" />
                  Open in IDE
                </DropdownMenuItem>
                {onEdit && (
                  <DropdownMenuItem
                    onClick={(e) => {
                      e.stopPropagation();
                      handleEdit(project);
                    }}
                  >
                    <Edit className="mr-2 h-4 w-4" />
                    Edit
                  </DropdownMenuItem>
                )}
                {fetchProjects && setError && (
                  <DropdownMenuItem
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(project.id, project.name);
                    }}
                    className="text-destructive"
                  >
                    <Trash2 className="mr-2 h-4 w-4" />
                    Delete
                  </DropdownMenuItem>
                )}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
        <CardDescription className="flex items-center">
          <Calendar className="mr-1 h-3 w-3" />
          Created {new Date(project.created_at).toLocaleDateString()}
        </CardDescription>
      </CardHeader>

      {sortedTasks.length > 0 && (
        <div className="px-6 pb-6">
          <div className="space-y-2">
            {displayedTasks.map((task) => (
              <ThreadListItem
                key={task.id}
                task={task}
                onClick={() => onThreadClick(project.id, task.id)}
              />
            ))}
          </div>

          {hasMore && !showAll && (
            <Button
              variant="ghost"
              size="sm"
              className="w-full mt-3 text-sm"
              onClick={() => setShowAll(true)}
            >
              <ChevronDown className="mr-1 h-4 w-4" />
              Show {sortedTasks.length - 5} more threads
            </Button>
          )}
        </div>
      )}
    </Card>
  );
}

export default ProjectWithThreads;
