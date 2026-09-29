/**
 * Tauri commands for communicating with the Rust backend
 * These commands are exposed from src-tauri/main.rs
 */

import { invoke } from '@tauri-apps/api';

// Session management

export interface InitVaultParams {
  name?: string;
  masterPassword: string;
}

export interface InitVaultResult {
  vaultId: string;
  vaultName: string;
}

export interface UnlockParams {
  vaultId?: string;
  masterPassword: string;
}

export interface UnlockResult {
  vaultId: string;
  vaultName: string;
}

export interface LockResult {
  success: boolean;
}

export interface SessionStatusResult {
  status: 'locked' | 'unlocked' | 'initializing';
  vaultId?: string;
  vaultName?: string;
}

// Item management

export interface AddItemParams {
  type: 'password' | 'note' | 'generic';
  name: string;
  service?: string;
  username?: string;
  password?: string;
  url?: string;
  content?: string;
  notes?: string;
}

export interface AddItemResult {
  itemId: string;
}

export interface GetItemParams {
  itemId: string;
}

export interface GetItemResult {
  item: Item;
}

export interface ListItemsParams {
  type?: 'password' | 'note' | 'generic';
  search?: string;
  limit?: number;
}

export interface ListItemsResult {
  items: Item[];
}

export interface RemoveItemParams {
  itemId: string;
  force?: boolean;
}

export interface RemoveItemResult {
  success: boolean;
}

// Vault management

export interface ListVaultsResult {
  vaults: VaultInfo[];
}

// Synchronication

export interface SyncStatusResult {
  isSyncing: boolean;
  lastSyncTime?: string;
  connectedDevices: number;
}

// Command functions

export const commands = {
  // Session
  initVault: (params: InitVaultParams) => invoke<InitVaultResult>('init_vault', params),
  unlock: (params: UnlockParams) => invoke<UnlockResult>('unlock', params),
  lock: () => invoke<LockResult>('lock'),
  getSessionStatus: () => invoke<SessionStatusResult>('get_session_status'),
  
  // Items
  addItem: (params: AddItemParams) => invoke<AddItemResult>('add_item', params),
  getItem: (params: GetItemParams) => invoke<GetItemResult>('get_item', params),
  listItems: (params?: ListItemsParams) => invoke<ListItemsResult>('list_items', params || {}),
  removeItem: (params: RemoveItemParams) => invoke<RemoveItemResult>('remove_item', params),
  
  // Vault
  listVaults: () => invoke<ListVaultsResult>('list_vaults'),
  
  // Sync
  getSyncStatus: () => invoke<SyncStatusResult>('get_sync_status'),
};

// Re-export Item type
export type { Item, VaultInfo, SessionStatus } from './types';
