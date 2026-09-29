/**
 * Type definitions for MLS frontend
 */

// Item types
export type ItemType = 'password' | 'note' | 'generic';

// Password item
export interface PasswordItem {
  type: 'password';
  id: string;
  name: string;
  service: string;
  username?: string;
  password: string;
  url?: string;
  notes?: string;
  updatedAt: string;
}

// Note item
export interface NoteItem {
  type: 'note';
  id: string;
  name: string;
  title?: string;
  content: string;
  updatedAt: string;
}

// Generic item
export interface GenericItem {
  type: 'generic';
  id: string;
  name: string;
  data: string;
  updatedAt: string;
}

// Union type for all items
export type Item = PasswordItem | NoteItem | GenericItem;

// Vault information
export interface VaultInfo {
  id: string;
  name: string;
  createdAt: string;
  itemCount: number;
}

// Session status
export type SessionStatus = 'locked' | 'unlocked' | 'initializing';

// API Response types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// Tauri command response
export interface CommandResponse<T> {
  result: T;
  error?: string;
}
