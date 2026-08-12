export type ProxySource = "system" | "pac" | "manual";
export type ProxyMode = "direct" | ProxySource;
export type ProxyType = "http" | "socks5";
export type ProxyRoute = "direct" | "proxy";
export type ProxyResolver = "direct" | "system" | "pac" | "manual" | "bypass";

export interface ProxySettings {
  mode: ProxyMode
  preferredMode: ProxySource
  pacUrl: string
  proxyType: ProxyType
  host: string
  port: number | null
  username: string
  bypass: string[]
  hasPassword: boolean
  warning?: string
}

export interface ProxySettingsInput {
  mode: ProxyMode
  preferredMode: ProxySource
  pacUrl: string
  proxyType: ProxyType
  host: string
  port: number | null
  username: string
  bypass: string[]
  password?: string | null
  clearPassword: boolean
}

export interface ProxyTestResult {
  success: boolean
  status?: number
  elapsedMs: number
  message: string
  route?: ProxyRoute
  resolver?: ProxyResolver
  fallbackAttempts?: number
}
