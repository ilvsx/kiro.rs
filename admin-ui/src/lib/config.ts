interface KiroConfig {
  basePath: string
}

declare global {
  interface Window {
    __KIRO_CONFIG__?: KiroConfig
  }
}

export function getConfig(): KiroConfig {
  return window.__KIRO_CONFIG__ || { basePath: '' }
}

export function getApiBaseUrl(): string {
  const { basePath } = getConfig()
  return `${basePath}/api/admin`
}
