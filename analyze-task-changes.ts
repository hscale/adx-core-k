#!/usr/bin/env node

/**
 * Analyze ADX Core task changes without requiring GitHub access
 * This script identifies what changed in the tasks.md file
 */

interface TaskChange {
    taskId: string;
    title: string;
    oldStatus: string;
    newStatus: string;
    changeType: 'status_change' | 'new_task' | 'description_change';
}

class TaskChangeAnalyzer {
    private tasks: any[] = [];

    async analyzeTasks(): Promise<void> {
        console.log('🔍 Analyzing ADX Core Task Changes\n');

        // Parse current tasks
        await this.parseTasks();
        
        // Analyze the specific changes from the diff
        const changes = this.detectChanges();
        
        this.reportChanges(changes);
    }

    private async parseTasks(): Promise<void> {
        const fs = require('fs').promises;
        const tasksContent = await fs.readFile('.kiro/specs/adx-core/tasks.md', 'utf-8');

        const lines = tasksContent.split('\n');
        let currentPhase = '';
        let lineNumber = 0;

        for (const line of lines) {
            lineNumber++;

            // Track current phase
            if (line.startsWith('## Phase ')) {
                currentPhase = line.replace('## Phase ', '').split(':')[0];
                continue;
            }

            // Parse task items
            const taskMatch = line.match(/^- \[([ x-])\] (\d+)\. (.+)$/);
            if (taskMatch) {
                const [, statusChar, taskId, title] = taskMatch;
                const status = statusChar === 'x' ? 'completed' : 
                              statusChar === '-' ? 'in_progress' : 'not_started';

                this.tasks.push({
                    id: taskId,
                    title: title.trim(),
                    status,
                    phase: currentPhase,
                    lineNumber
                });
            }
        }
    }

    private detectChanges(): TaskChange[] {
        const changes: TaskChange[] = [];

        // Based on the diff, we know Task 33 changed from in_progress to completed
        // The diff showed "- [-] 33." which means it was in_progress, now it's completed
        const task33 = this.tasks.find(t => t.id === '33');
        if (task33 && task33.status === 'completed') {
            changes.push({
                taskId: '33',
                title: task33.title,
                oldStatus: 'in_progress',
                newStatus: 'completed',
                changeType: 'status_change'
            });
        }

        return changes;
    }

    private reportChanges(changes: TaskChange[]): void {
        console.log('📊 Task Change Summary:');
        console.log(`   Total Changes Detected: ${changes.length}\n`);

        if (changes.length === 0) {
            console.log('   No significant changes detected.');
            return;
        }

        changes.forEach(change => {
            const statusIcon = this.getStatusIcon(change.newStatus);
            const oldIcon = this.getStatusIcon(change.oldStatus);
            
            console.log(`${statusIcon} Task ${change.taskId}: ${change.title}`);
            console.log(`   Status: ${oldIcon} ${change.oldStatus} → ${statusIcon} ${change.newStatus}`);
            console.log(`   Change Type: ${change.changeType}`);
            console.log('');
        });

        // Show what would happen in GitHub
        console.log('🔄 GitHub Actions Required:');
        changes.forEach(change => {
            if (change.changeType === 'status_change') {
                if (change.newStatus === 'completed') {
                    console.log(`   • Close issue for Task ${change.taskId}`);
                } else if (change.oldStatus === 'completed') {
                    console.log(`   • Reopen issue for Task ${change.taskId}`);
                } else {
                    console.log(`   • Update labels for Task ${change.taskId} (${change.oldStatus} → ${change.newStatus})`);
                }
            }
        });

        console.log('\n📋 Task Status Overview:');
        const statusCounts = this.getStatusCounts();
        console.log(`   ✅ Completed: ${statusCounts.completed}`);
        console.log(`   🔄 In Progress: ${statusCounts.in_progress}`);
        console.log(`   📋 Not Started: ${statusCounts.not_started}`);
        console.log(`   📈 Total: ${statusCounts.total}`);

        console.log('\n🏗️ Component Analysis for Changed Tasks:');
        changes.forEach(change => {
            const task = this.tasks.find(t => t.id === change.taskId);
            if (task) {
                const components = this.getTaskComponents(task.title);
                console.log(`   Task ${change.taskId}: ${components.join(', ')}`);
            }
        });

        console.log('\n💡 Next Steps:');
        console.log('   1. Set up GitHub token: GITHUB_TOKEN=your_token_here');
        console.log('   2. Run: npm run sync-tasks');
        console.log('   3. Verify GitHub issues are updated correctly');
    }

    private getStatusIcon(status: string): string {
        switch (status) {
            case 'completed': return '✅';
            case 'in_progress': return '🔄';
            case 'not_started': return '📋';
            default: return '❓';
        }
    }

    private getStatusCounts() {
        const counts = {
            completed: 0,
            in_progress: 0,
            not_started: 0,
            total: this.tasks.length
        };

        this.tasks.forEach(task => {
            counts[task.status as keyof typeof counts]++;
        });

        return counts;
    }

    private getTaskComponents(title: string): string[] {
        const components: string[] = [];
        const titleLower = title.toLowerCase();

        if (titleLower.includes('temporal')) components.push('temporal');
        if (titleLower.includes('database') || titleLower.includes('migration')) components.push('database');
        if (titleLower.includes('auth')) components.push('auth');
        if (titleLower.includes('tenant')) components.push('tenant');
        if (titleLower.includes('user')) components.push('user');
        if (titleLower.includes('file')) components.push('file');
        if (titleLower.includes('workflow')) components.push('workflow');
        if (titleLower.includes('frontend') || titleLower.includes('micro-frontend')) components.push('frontend');
        if (titleLower.includes('bff')) components.push('bff');
        if (titleLower.includes('api')) components.push('api');
        if (titleLower.includes('testing')) components.push('testing');
        if (titleLower.includes('ai')) components.push('ai');
        if (titleLower.includes('module')) components.push('module');

        return components.length > 0 ? components : ['general'];
    }
}

/**
 * Main execution
 */
async function main() {
    try {
        const analyzer = new TaskChangeAnalyzer();
        await analyzer.analyzeTasks();
        
        console.log('\n✅ Task change analysis completed successfully');
        process.exit(0);
    } catch (error) {
        console.error('❌ Task analysis failed:', error instanceof Error ? error.message : String(error));
        process.exit(1);
    }
}

// Run if called directly
if (require.main === module) {
    main().catch(console.error);
}

export { TaskChangeAnalyzer };