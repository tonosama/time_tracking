import { invoke } from '@tauri-apps/api/core';

const logToFile = (level: string, message: string) => {
  const logMessage = `[${level.toUpperCase()}] ${message}`;
  invoke('log_to_file', { message: logMessage }).catch(console.error);
};

export const logger = {
  info: (message: string) => {
    console.log(message);
    logToFile('info', message);
  },
  warn: (message: string) => {
    console.warn(message);
    logToFile('warn', message);
  },
  error: (message: string) => {
    console.error(message);
    logToFile('error', message);
  },
};

// Loggerクラスを追加
export class Logger {
  static debug(component: string, message: string, data?: any) {
    const logMessage = `[${component}] ${message}`;
    console.debug(logMessage, data);
    logToFile('debug', logMessage);
  }

  static info(component: string, message: string, data?: any) {
    const logMessage = `[${component}] ${message}`;
    console.info(logMessage, data);
    logToFile('info', logMessage);
  }

  static warn(component: string, message: string, data?: any) {
    const logMessage = `[${component}] ${message}`;
    console.warn(logMessage, data);
    logToFile('warn', logMessage);
  }

  static error(component: string, message: string, data?: any) {
    const logMessage = `[${component}] ${message}`;
    console.error(logMessage, data);
    logToFile('error', logMessage);
  }

  static performance(component: string, action: string, duration: number, data?: any) {
    const logMessage = `[${component}] ${action} - ${duration.toFixed(2)}ms`;
    console.log(logMessage, data);
    logToFile('info', logMessage);
  }

  static userAction(component: string, action: string, data?: any) {
    const logMessage = `[${component}] User Action: ${action}`;
    console.log(logMessage, data);
    logToFile('info', logMessage);
  }

  static apiCall(component: string, method: string, endpoint: string, data?: any) {
    const logMessage = `[${component}] API ${method} ${endpoint}`;
    console.log(logMessage, data);
    logToFile('info', logMessage);
  }

  static apiSuccess(component: string, method: string, endpoint: string, data?: any) {
    const logMessage = `[${component}] API ${method} ${endpoint} - Success`;
    console.log(logMessage, data);
    logToFile('info', logMessage);
  }

  static apiError(component: string, method: string, endpoint: string, error: any, data?: any) {
    const logMessage = `[${component}] API ${method} ${endpoint} - Error`;
    console.error(logMessage, error, data);
    logToFile('error', logMessage);
  }
}
