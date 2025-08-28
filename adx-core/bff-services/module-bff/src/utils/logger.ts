import winston from 'winston';

const logLevel = process.env.LOG_LEVEL || 'info';
const logFormat = process.env.LOG_FORMAT || 'json';

const createLogger = (service: string) => {
  const formats = [
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json(),
  ];

  if (process.env.NODE_ENV !== 'production' || logFormat === 'simple') {
    formats.push(
      winston.format.colorize(),
      winston.format.simple()
    );
  }

  return winston.createLogger({
    level: logLevel,
    format: winston.format.combine(...formats),
    defaultMeta: { service },
    transports: [
      new winston.transports.Console(),
    ],
  });
};

export { createLogger };