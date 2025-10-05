import { useCallback, useEffect, useState, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { AlertTriangle, Plus } from 'lucide-react';
import { Loader } from '@/components/ui/loader';
import { projectsApi, attemptsApi } from '@/lib/api';
import { openTaskForm } from '@/lib/openTaskForm';

import { useSearch } from '@/contexts/search-context';
import { useQuery } from '@tanstack/react-query';
import { useTaskViewManager } from '@/hooks/useTaskViewManager';
import {
  useKeyCreate,
  useKeyExit,
  useKeyFocusSearch,
  Scope,
  useKeyDeleteTask,
} from '@/keyboard';

import { TaskDetailsPanel } from '@/components/tasks/TaskDetailsPanel';
import { ThreadList } from '@/components/tasks/ThreadList';
import type { TaskWithAttemptStatus, Project, TaskAttempt } from 'shared/types';
import { useProjectTasks } from '@/hooks/useProjectTasks';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import NiceModal from '@ebay/nice-modal-react';
import { useHotkeysContext } from 'react-hotkeys-hook';

type Task = TaskWithAttemptStatus;

export function ProjectTasks() {
  const { t } = useTranslation(['tasks', 'common']);
  const { projectId, taskId, attemptId } = useParams<{
    projectId: string;
    taskId?: string;
    attemptId?: string;
  }>();
  const navigate = useNavigate();
  const { enableScope, disableScope } = useHotkeysContext();

  useEffect(() => {
    enableScope(Scope.KANBAN);

    return () => {
      disableScope(Scope.KANBAN);
    };
  }, [enableScope, disableScope]);

  const [project, setProject] = useState<Project | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Helper functions to open task forms
  const handleCreateTask = () => {
    if (project?.id) {
      openTaskForm({ projectId: project.id });
    }
  };

  const handleEditTask = (task: Task) => {
    if (project?.id) {
      openTaskForm({ projectId: project.id, task });
    }
  };

  const { query: searchQuery, focusInput } = useSearch();

  // Panel state
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [isPanelOpen, setIsPanelOpen] = useState(false);

  // Navigation helpers
  const { navigateToTask, navigateToAttempt } = useTaskViewManager();

  // Attempts fetching (only when task is selected)
  const { data: attempts = [] } = useQuery({
    queryKey: ['taskAttempts', selectedTask?.id],
    queryFn: () => attemptsApi.getAll(selectedTask!.id),
    enabled: !!selectedTask?.id,
    refetchInterval: 5000,
  });

  // Selected attempt logic
  const selectedAttempt = useMemo(() => {
    if (!attempts.length) return null;
    if (attemptId) {
      const found = attempts.find((a) => a.id === attemptId);
      if (found) return found;
    }
    return attempts[0] || null; // Most recent fallback
  }, [attempts, attemptId]);

  // Navigation callback for attempt selection
  const setSelectedAttempt = useCallback(
    (attempt: TaskAttempt | null) => {
      if (!selectedTask) return;

      if (attempt) {
        navigateToAttempt(projectId!, selectedTask.id, attempt.id);
      } else {
        navigateToTask(projectId!, selectedTask.id);
      }
    },
    [navigateToTask, navigateToAttempt, projectId, selectedTask]
  );

  // Stream tasks for this project
  const {
    tasks,
    tasksById,
    isLoading,
    error: streamError,
  } = useProjectTasks(projectId || '');

  // Sync selectedTask with URL params and live task updates
  useEffect(() => {
    if (taskId) {
      const t = taskId ? tasksById[taskId] : undefined;
      if (t) {
        setSelectedTask(t);
        setIsPanelOpen(true);
      }
    } else {
      setSelectedTask(null);
      setIsPanelOpen(false);
    }
  }, [taskId, tasksById]);

  // Define task creation handler
  const handleCreateNewTask = useCallback(() => {
    handleCreateTask();
  }, [handleCreateTask]);

  // Semantic keyboard shortcuts for kanban page
  // Prevent default is needed to stop the input having the value 'c'
  useKeyCreate(handleCreateNewTask, {
    scope: Scope.KANBAN,
    preventDefault: true,
  });

  useKeyFocusSearch(
    () => {
      focusInput();
    },
    {
      scope: Scope.KANBAN,
      preventDefault: true, // Prevent Firefox quick find
    }
  );

  useKeyExit(
    () => {
      if (isPanelOpen) {
        handleClosePanel();
      } else {
        navigate('/projects');
      }
    },
    { scope: Scope.KANBAN }
  );

  // Memoize filtered tasks based on search query and sort by most recent
  const filteredTasks = useMemo(() => {
    let filtered = tasks;
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = tasks.filter(
        (task) =>
          task.title.toLowerCase().includes(query) ||
          (task.description && task.description.toLowerCase().includes(query))
      );
    }
    // Sort by most recent updated_at
    return [...filtered].sort((a, b) => {
      const aTime = new Date(a.updated_at || a.created_at).getTime();
      const bTime = new Date(b.updated_at || b.created_at).getTime();
      return bTime - aTime;
    });
  }, [tasks, searchQuery]);

  // Delete task shortcut
  useKeyDeleteTask(
    () => {
      if (selectedTask) {
        handleDeleteTask(selectedTask.id);
      }
    },
    {
      scope: Scope.KANBAN,
      preventDefault: true,
    }
  );

  const fetchProject = useCallback(async () => {
    try {
      const result = await projectsApi.getById(projectId!);
      setProject(result);
    } catch (err) {
      setError('Failed to load project');
    }
  }, [projectId]);

  const handleClosePanel = useCallback(() => {
    // setIsPanelOpen(false);
    // setSelectedTask(null);
    // Remove task ID from URL when closing panel
    navigate(`/projects/${projectId}/tasks`, { replace: true });
  }, [projectId, navigate]);

  const handleDeleteTask = useCallback(
    (taskId: string) => {
      const task = tasksById[taskId];
      if (task) {
        NiceModal.show('delete-task-confirmation', {
          task,
          projectId: projectId!,
        })
          .then(() => {
            // Task was deleted, close panel if this task was selected
            if (selectedTask?.id === taskId) {
              handleClosePanel();
            }
          })
          .catch(() => {
            // Modal was cancelled - do nothing
          });
      }
    },
    [tasksById, projectId, selectedTask, handleClosePanel]
  );

  const handleEditTaskCallback = useCallback(
    (task: Task) => {
      handleEditTask(task);
    },
    [handleEditTask]
  );

  const handleViewTaskDetails = useCallback(
    (task: Task, attemptIdToShow?: string) => {
      if (attemptIdToShow) {
        navigateToAttempt(projectId!, task.id, attemptIdToShow);
      } else {
        navigateToTask(projectId!, task.id);
      }
    },
    [projectId, navigateToTask, navigateToAttempt]
  );

  // Initialize project when projectId changes
  useEffect(() => {
    if (projectId) {
      fetchProject();
    }
  }, [projectId, fetchProject]);

  // Remove legacy direct-navigation handler; live sync above covers this

  if (isLoading) {
    return <Loader message={t('loading')} size={32} className="py-8" />;
  }

  if (error) {
    return (
      <div className="p-4">
        <Alert>
          <AlertTitle className="flex items-center gap-2">
            <AlertTriangle size="16" />
            {t('common:states.error')}
          </AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div className="min-h-full">
      {streamError && (
        <Alert className="w-full z-30 sticky top-0">
          <AlertTitle className="flex items-center gap-2">
            <AlertTriangle size="16" />
            {t('common:states.reconnecting')}
          </AlertTitle>
          <AlertDescription>{streamError}</AlertDescription>
        </Alert>
      )}

      {/* Show thread list or detail panel */}
      {isPanelOpen && selectedTask ? (
        <TaskDetailsPanel
          task={selectedTask}
          projectHasDevScript={!!project?.dev_script}
          projectId={projectId!}
          onClose={handleClosePanel}
          onEditTask={handleEditTaskCallback}
          onDeleteTask={handleDeleteTask}
          onNavigateToTask={(taskId) => {
            const task = tasksById[taskId];
            if (task) {
              handleViewTaskDetails(task);
            }
          }}
          selectedAttempt={selectedAttempt}
          attempts={attempts}
          setSelectedAttempt={setSelectedAttempt}
          tasksById={tasksById}
        />
      ) : (
        <div className="max-w-5xl mx-auto px-4 py-6">
          {tasks.length === 0 ? (
            <Card>
              <CardContent className="text-center py-8">
                <p className="text-muted-foreground">{t('empty.noTasks')}</p>
                <Button className="mt-4" onClick={handleCreateNewTask}>
                  <Plus className="h-4 w-4 mr-2" />
                  {t('empty.createFirst')}
                </Button>
              </CardContent>
            </Card>
          ) : (
            <ThreadList
              tasks={filteredTasks}
              onTaskClick={handleViewTaskDetails}
              loading={isLoading}
            />
          )}
        </div>
      )}
    </div>
  );
}
