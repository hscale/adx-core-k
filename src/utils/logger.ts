/**
 * Simple logger utility for the GitHub sync service
 */
export class Logger {
  private formatMessage(level: string, message: string, ...args: unknown[]): string {
    const timestamp = new Date().toISOString();
    const formattedArgs = args.length > 0 ? ' ' + args.map(arg => 
      typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
    ).join(' ') : '';
    
    return `[${timestamp}] ${level.toUpperCase()}: ${message}${formattedArgs}`;
  }

  info(message: string, ...args: unknown[]): void {
    console.log(this.formatMessage('info', message, ...args));
  }

  warn(message: string, ...args: unknown[]): void {
    console.warn(this.formatMessage('warn', message, ...args));
  }

  error(message: string, ...args: unknown[]): void {
    console.error(this.formatMessage('error', message, ...args));
  }

  debug(message: string, ...args: unknown[]): void {
    if (process.env['DEBUG'] || process.env['NODE_ENV'] === 'development') {
      console.debug(this.formatMessage('debug', message, ...args));
    }
  }
}

export const logger = new Logger();