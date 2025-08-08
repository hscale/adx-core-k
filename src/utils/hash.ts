import { createHash } from 'crypto';
import { KiroTask } from '../config/types';

/**
 * Generate a hash for a task to detect changes
 */
export function hashTask(task: KiroTask): string {
  const hashData = {
    title: task.title,
    description: task.description || '',
    status: task.status,
    requirements: task.requirements || [],
  };
  
  return createHash('md5')
    .update(JSON.stringify(hashData))
    .digest('hex');
}

/**
 * Generate a hash for file content
 */
export function hashContent(content: string): string {
  return createHash('md5')
    .update(content)
    .digest('hex');
}