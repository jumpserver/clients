export type ProxyMode = "direct" | "manual";
export type ProxyType = "http" | "socks5";

export interface ProxySettings {
  mode: ProxyMode
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
}
