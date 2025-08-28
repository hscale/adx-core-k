#!/usr/bin/env node

import { GitHubClient } from './src/services/GitHubClient';
import { ConfigManager } from './src/config/ConfigManager';
import { logger } from './src/utils/logger';
import { KiroTask } from './src/config/types';

/**
 * Comprehensive GitHub Task Sync Handler for ADX Core Tasks
 * Analyzes the tasks.md file and syncs all tasks with GitHub issues
 */
class ADXCoreTaskSync {
    private configManager: ConfigManager;
    private tasks: KiroTask[] = [];

    constructor() {
        this.configManager = new ConfigManager();
    }

    async syncAllTasks(dryRun: boolean = false): Promise<void> {
        try {
            logger.info('Starting comprehensive ADX Core task sync to GitHub', { dryRun });

            // Parse tasks from the tasks.md file first
            await this.parseTasks();
            logger.info(`Found ${this.tasks.length} tasks to sync`);

            if (dryRun) {
                // In dry run mode, just analyze and report what would be done
                await this.performDryRun();
                return;
            }

            // Load GitHub configuration
            const config = await this.configManager.load();
            if (!config.enabled) {
                logger.warn('GitHub sync is disabled, skipping sync');
                return;
            }

            // Initialize GitHub client
            const githubClient = new GitHubClient(config);

            // Test connection first
            const connectionTest = await githubClient.testConnection();
            if (!connectionTest.success) {
                throw new Error(`GitHub connection failed: ${connectionTest.message}`);
            }

            // Sync each task
            let syncedCount = 0;
            let errorCount = 0;

            for (const task of this.tasks) {
                try {
                    await this.syncTask(task, githubClient, config);
                    syncedCount++;
                    logger.info(`Synced task ${task.id}: ${task.title}`);
                } catch (error) {
                    errorCount++;
                    logger.error(`Failed to sync task ${task.id}`, {
                        error: error instanceof Error ? error.message : String(error)
                    });
                }
            }

            logger.info('ADX Core task sync completed', {
                totalTasks: this.tasks.length,
                synced: syncedCount,
                errors: errorCount
            });

        } catch (error) {
            logger.error('Failed to sync ADX Core tasks', {
                error: error instanceof Error ? error.message : String(error)
            });
            throw error;
        }
    }

    private async performDryRun(): Promise<void> {
        console.log('\n🔍 DRY RUN MODE - Analyzing ADX Core Tasks\n');

        // Group tasks by status
        const completed = this.tasks.filter(t => t.status === 'completed');
        const inProgress = this.tasks.filter(t => t.status === 'in_progress');
        const notStarted = this.tasks.filter(t => t.status === 'not_started');

        console.log(`📊 Task Status Summary:`);
        console.log(`   ✅ Completed: ${completed.length}`);
        console.log(`   🔄 In Progress: ${inProgress.length}`);
        console.log(`   📋 Not Started: ${notStarted.length}`);
        console.log(`   📈 Total: ${this.tasks.length}`);

        // Show completed tasks
        if (completed.length > 0) {
            console.log(`\n✅ Completed Tasks (would be closed issues):`);
            completed.forEach(task => {
                console.log(`   ${task.id}: ${task.title}`);
            });
        }

        // Show in-progress tasks
        if (inProgress.length > 0) {
            console.log(`\n🔄 In Progress Tasks (would be open issues):`);
            inProgress.forEach(task => {
                console.log(`   ${task.id}: ${task.title}`);
            });
        }

        // Show not started tasks
        if (notStarted.length > 0) {
            console.log(`\n📋 Not Started Tasks (would be open issues):`);
            notStarted.slice(0, 10).forEach(task => {
                console.log(`   ${task.id}: ${task.title}`);
            });
            if (notStarted.length > 10) {
                console.log(`   ... and ${notStarted.length - 10} more`);
            }
        }

        // Show component breakdown
        const componentCounts: Record<string, number> = {};
        this.tasks.forEach(task => {
            const labels = this.generateLabels(task, { labelPrefix: 'adx-core:' });
            labels.forEach(label => {
                if (label.startsWith('component:')) {
                    const component = label.replace('component:', '');
                    componentCounts[component] = (componentCounts[component] || 0) + 1;
                }
            });
        });

        console.log(`\n🏗️ Component Breakdown:`);
        Object.entries(componentCounts)
            .sort(([, a], [, b]) => b - a)
            .forEach(([component, count]) => {
                console.log(`   ${component}: ${count} tasks`);
            });

        // Show phase breakdown
        const phaseCounts: Record<string, number> = {};
        this.tasks.forEach(task => {
            const labels = this.generateLabels(task, { labelPrefix: 'adx-core:' });
            labels.forEach(label => {
                if (label.startsWith('phase:')) {
                    const phase = label.replace('phase:', '');
                    phaseCounts[phase] = (phaseCounts[phase] || 0) + 1;
                }
            });
        });

        console.log(`\n📅 Phase Breakdown:`);
        Object.entries(phaseCounts)
            .sort(([a], [b]) => a.localeCompare(b))
            .forEach(([phase, count]) => {
                console.log(`   Phase ${phase}: ${count} tasks`);
            });

        console.log(`\n💡 To perform actual sync, run without --dry-run flag`);
        console.log(`   Ensure GITHUB_TOKEN and GITHUB_REPOSITORY are set`);
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

            // Parse task items - handle both simple numbers (1.) and complex numbers (1.1, 10.1, etc.)
            // Also handle Windows line endings by trimming the line first
            const trimmedLine = line.trim();
            const taskMatch = trimmedLine.match(/^- \[([ x-])\] (\d+(?:\.\d+)*)\. (.+)$/);
            if (taskMatch) {
                const [, statusChar, taskId, title] = taskMatch;
                const status = statusChar === 'x' ? 'completed' : 
                              statusChar === '-' ? 'in_progress' : 'not_started';

                // Extract requirements from the task description
                const requirements = this.extractRequirements(lines, lineNumber);

                this.tasks.push({
                    id: taskId,
                    title: title.trim(),
                    status,
                    filePath: '.kiro/specs/adx-core/tasks.md',
                    lineNumber,
                    specName: 'adx-core',
                    requirements,
                    description: this.extractTaskDescription(lines, lineNumber)
                });
            }
        }
    }

    private extractRequirements(lines: string[], startLine: number): string[] {
        const requirements: string[] = [];

        // Look for requirements in the next few lines
        for (let i = startLine; i < Math.min(startLine + 10, lines.length); i++) {
            const line = lines[i];
            const reqMatch = line.match(/_Requirements: ([^_]+)_/);
            if (reqMatch) {
                const reqText = reqMatch[1];
                const reqs = reqText.split(',').map(r => r.trim().replace(/[()]/g, ''));
                requirements.push(...reqs);
                break;
            }
        }

        return requirements;
    }

    private extractTaskDescription(lines: string[], startLine: number): string {
        const description: string[] = [];
        let inTaskDescription = false;

        // Look for task description in the next lines until we hit another task or section
        for (let i = startLine; i < Math.min(startLine + 20, lines.length); i++) {
            const line = lines[i];

            // Stop if we hit another task or major section
            if (line.match(/^- \[([ x])\] \d+\./) || line.startsWith('## ') || line.startsWith('# ')) {
                if (i > startLine) break;
            }

            // Start collecting after the task line
            if (i > startLine) {
                if (line.trim().startsWith('- ') && !line.match(/^- \[([ x])\] \d+\./)) {
                    description.push(line.trim());
                    inTaskDescription = true;
                } else if (inTaskDescription && line.trim() === '') {
                    break;
                } else if (inTaskDescription && line.trim().startsWith('_Requirements:')) {
                    break;
                }
            }
        }

        return description.join('\n');
    }

    private async syncTask(task: KiroTask, githubClient: GitHubClient, config: any): Promise<void> {
        // Check if issue already exists
        const label = `${config.labelPrefix}${task.id}`;
        const existingIssue = await githubClient.findIssueByLabel(label);

        if (existingIssue) {
            await this.updateExistingIssue(existingIssue, task, githubClient, config);
        } else {
            await this.createNewIssue(task, githubClient, config);
        }
    }

    private async updateExistingIssue(
        existingIssue: any,
        task: KiroTask,
        githubClient: GitHubClient,
        config: any
    ): Promise<void> {
        const title = this.generateIssueTitle(task);
        const body = this.generateIssueDescription(task);
        const labels = this.generateLabels(task, config);

        // Update issue content
        await githubClient.updateIssue(existingIssue.number, title, body);

        // Update labels
        await githubClient.updateIssueLabels(existingIssue.number, labels);

        // Handle status changes
        if (task.status === 'completed' && existingIssue.state === 'open') {
            await githubClient.closeIssue(existingIssue.number);
            logger.info('Closed completed task issue', {
                taskId: task.id,
                issueNumber: existingIssue.number
            });
        } else if (task.status !== 'completed' && existingIssue.state === 'closed') {
            // Reopen if task is no longer completed
            await githubClient.reopenIssue(existingIssue.number);
            logger.info('Reopened task issue', {
                taskId: task.id,
                issueNumber: existingIssue.number
            });
        }

        logger.info('Updated GitHub issue for task', {
            taskId: task.id,
            issueNumber: existingIssue.number,
            issueUrl: existingIssue.html_url
        });
    }

    private async createNewIssue(
        task: KiroTask,
        githubClient: GitHubClient,
        config: any
    ): Promise<void> {
        const title = this.generateIssueTitle(task);
        const body = this.generateIssueDescription(task);
        const labels = this.generateLabels(task, config);

        const issue = await githubClient.createIssue(title, body, labels);

        // Close immediately if task is completed
        if (task.status === 'completed') {
            await githubClient.closeIssue(issue.number);
        }

        logger.info('Created GitHub issue for task', {
            taskId: task.id,
            issueNumber: issue.number,
            issueUrl: issue.html_url,
            status: task.status
        });
    }

    private generateIssueTitle(task: KiroTask): string {
        const statusIcon = task.status === 'completed' ? '✅' :
            task.status === 'in_progress' ? '🔄' : '📋';
        return `${statusIcon} [${task.specName}] ${task.id}: ${task.title}`;
    }

    private generateLabels(task: KiroTask, config: any): string[] {
        const labels = [
            `${config.labelPrefix}${task.id}`,
            `spec:${task.specName}`,
            `status:${task.status}`,
        ];

        // Add phase label based on task ID
        const taskNum = parseInt(task.id);
        if (taskNum <= 7) labels.push('phase:1-2');
        else if (taskNum <= 13) labels.push('phase:3');
        else if (taskNum <= 18) labels.push('phase:4');
        else if (taskNum <= 21) labels.push('phase:5');
        else if (taskNum <= 25) labels.push('phase:6');
        else if (taskNum <= 27) labels.push('phase:7');
        else if (taskNum <= 31) labels.push('phase:8');
        else if (taskNum <= 35) labels.push('phase:9');
        else if (taskNum <= 37) labels.push('phase:10');
        else if (taskNum <= 40) labels.push('phase:11');
        else labels.push('phase:12');

        // Add requirement labels
        if (task.requirements) {
            task.requirements.forEach((req: string) => {
                if (req.match(/^\d+\.\d+$/)) {
                    labels.push(`requirement:${req}`);
                }
            });
        }

        // Add component labels based on task content
        const titleLower = task.title.toLowerCase();
        if (titleLower.includes('temporal')) labels.push('component:temporal');
        if (titleLower.includes('database') || titleLower.includes('migration')) labels.push('component:database');
        if (titleLower.includes('auth')) labels.push('component:auth');
        if (titleLower.includes('tenant')) labels.push('component:tenant');
        if (titleLower.includes('user')) labels.push('component:user');
        if (titleLower.includes('file')) labels.push('component:file');
        if (titleLower.includes('workflow')) labels.push('component:workflow');
        if (titleLower.includes('frontend') || titleLower.includes('micro-frontend')) labels.push('component:frontend');
        if (titleLower.includes('bff')) labels.push('component:bff');
        if (titleLower.includes('api')) labels.push('component:api');
        if (titleLower.includes('testing')) labels.push('component:testing');
        if (titleLower.includes('ai')) labels.push('component:ai');
        if (titleLower.includes('module')) labels.push('component:module');

        return labels;
    }

    private generateIssueDescription(task: KiroTask): string {
        const statusEmoji = task.status === 'completed' ? '✅' :
            task.status === 'in_progress' ? '🔄' : '📋';

        return `## ${task.title}

${task.description || 'No detailed description available.'}

**Status:** ${statusEmoji} ${task.status.replace('_', ' ').toUpperCase()}

**Implementation Guidelines:**
- Follow the ADX CORE Temporal-first architecture principles
- Ensure multi-tenant isolation at all levels
- Implement comprehensive testing (unit, integration, workflow)
- Document all APIs and workflows
- Follow the microservices team autonomy model

**Architecture Requirements:**
${task.requirements ? task.requirements.map(req => `- ${req} (${this.getRequirementDescription(req)})`).join('\n') : '- No specific requirements listed'}

---
**Kiro Task Information**

- **Task ID:** ${task.id}
- **Spec:** ${task.specName}
- **Status:** ${task.status}
- **Source:** ${task.filePath}:${task.lineNumber}
- **Last Updated:** ${new Date().toISOString()}

*This issue was automatically synced by Kiro GitHub Task Sync*`;
    }

    private getRequirementDescription(requirement: string): string {
        const descriptions: Record<string, string> = {
            '1.1': 'Authentication and authorization system',
            '1.3': 'Security and compliance',
            '1.4': 'Data protection and privacy',
            '2.1': 'Multi-tenant architecture',
            '2.2': 'Tenant isolation and security',
            '2.3': 'Tenant management and billing',
            '3.1': 'Temporal-first backend microservices',
            '4.1': 'File management and storage',
            '4.2': 'File processing and workflows',
            '5.1': 'License management',
            '5.2': 'Quota enforcement',
            '5.3': 'Billing integration',
            '5.4': 'Subscription management',
            '6.1': 'Temporal-first API gateway and integration',
            '7.1': 'DevOps and operational excellence',
            '8.1': 'Frontend microservices architecture',
            '8.1.1': 'BFF pattern integration',
            '9.1': 'Multi-language support',
            '9.2': 'Internationalization',
            '9.3': 'Theming system',
            '9.4': 'Accessibility compliance',
            '9.5': 'RTL language support',
            '9.6': 'Locale-specific formatting',
            '10.1': 'Module system architecture',
            '10.2': 'Module marketplace',
            '10.3': 'Module sandboxing',
            '10.5': 'Module hot-loading',
            '10.8': 'Module development SDK',
            '10.9': 'Module documentation',
            '11.1': 'Temporal-first hybrid AI workflow orchestration',
            '11.2': 'AI service integration',
            '11.3': 'AI workflow activities',
            '12.1': 'Temporal-first white-label and custom domains',
            '13.1': 'Team autonomy and vertical ownership',
            '14.1': 'Cross-service workflow orchestration',
            '15.1': 'Module Federation and micro-frontend integration',
        };
        return descriptions[requirement] || requirement;
    }
}

/**
 * Main execution function
 */
async function main() {
    try {
        const dryRun = process.argv.includes('--dry-run') || process.argv.includes('-d');

        const syncHandler = new ADXCoreTaskSync();
        await syncHandler.syncAllTasks(dryRun);

        if (dryRun) {
            console.log('✅ ADX Core task analysis completed successfully');
        } else {
            console.log('✅ ADX Core task sync completed successfully');
        }
        process.exit(0);
    } catch (error) {
        console.error('❌ ADX Core task sync failed:', error instanceof Error ? error.message : String(error));
        process.exit(1);
    }
}

// Run if called directly
if (require.main === module) {
    main().catch(console.error);
}

export { ADXCoreTaskSync };