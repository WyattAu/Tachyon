/// <reference types="bun-types" />

declare global {
  interface Window {
    Tachyon: import('./src/index').TachyonAppState;
    TACHYON_API_URL?: string;
    htmx: any;
  }
}

export {};
