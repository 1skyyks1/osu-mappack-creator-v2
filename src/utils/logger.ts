import { invoke } from '@tauri-apps/api/core'

type LogLevel = 'INFO' | 'WARN' | 'ERROR'

const serializeError = (error: unknown) => {
  if (error instanceof Error) {
    return `${error.message}${error.stack ? ` | ${error.stack}` : ''}`
  }
  if (typeof error === 'string') {
    return error
  }
  try {
    return JSON.stringify(error)
  } catch {
    return String(error)
  }
}

const writeLog = async (level: LogLevel, message: string) => {
  await invoke('write_app_log', { level, message })
}

export const logInfo = async (message: string) => {
  console.info(message)
  try {
    await writeLog('INFO', message)
  } catch (error) {
    console.error('Failed to persist info log', error)
  }
}

export const logError = async (message: string, error?: unknown) => {
  const composed = error ? `${message} | ${serializeError(error)}` : message
  console.error(message, error)
  try {
    await writeLog('ERROR', composed)
  } catch (invokeError) {
    console.error('Failed to persist error log', invokeError)
  }
}

