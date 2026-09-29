/**
 * Unlocked Screen - Main application with item list
 */

import { useState, useEffect } from 'react';
import { commands, Item, VaultInfo } from '../commands';
import ItemList from '../components/ItemList';
import AddItemModal from '../components/AddItemModal';
import SettingsModal from '../components/SettingsModal';

interface UnlockedScreenProps {
  vaultId: string;
  vaultName: string;
  onLock: () => void;
}

function UnlockedScreen({ vaultId, vaultName, onLock }: UnlockedScreenProps) {
  const [items, setItems] = useState<Item[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  // Load items on mount
  const loadItems = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await commands.listItems({ search: searchQuery || undefined });
      setItems(result.items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load items');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadItems();
  }, [searchQuery]);

  // Refresh items
  const handleRefresh = () => {
    loadItems();
  };

  // Handle lock
  const handleLock = async () => {
    try {
      await commands.lock();
      onLock();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to lock vault');
    }
  };

  // Handle add item
  const handleAddItem = async (itemType: 'password' | 'note' | 'generic') => {
    setShowAddModal(true);
  };

  // Handle item added
  const handleItemAdded = () => {
    setShowAddModal(false);
    loadItems();
  };

  return (
    <div className="min-h-screen bg-gray-900 flex flex-col">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-4 py-3">
        <div className="flex items-center justify-between max-w-6xl mx-auto">
          <div className="flex items-center space-x-4">
            <h1 className="text-xl font-bold">{vaultName}</h1>
            <span className="text-gray-400">|</span>
            <span className="text-sm text-gray-400">Vault ID: {vaultId.slice(0, 8)}...</span>
          </div>
          
          <div className="flex items-center space-x-2">
            <div className="relative">
              <input
                type="text"
                placeholder="Search..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="input pl-10 w-64"
              />
              <svg
                className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.35-4.35" />
              </svg>
            </div>
            
            <button
              onClick={() => setShowSettings(true)}
              className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-md"
              title="Settings"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V15a1.65 1.65 0 0 0 1.51 1z" />
              </svg>
            </button>
            
            <button
              onClick={handleLock}
              className="btn btn-secondary px-4 py-2"
            >
              Lock
            </button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 max-w-6xl mx-auto w-full p-4">
        {error && (
          <div className="bg-red-600 text-white px-4 py-2 rounded-md mb-4">
            {error}
          </div>
        )}

        {loading ? (
          <div className="flex items-center justify-center h-64">
            <span className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></span>
          </div>
        ) : (
          <>
            {items.length === 0 ? (
              <div className="text-center py-12">
                <div className="text-6xl mb-4">🔒</div>
                <p className="text-gray-400 text-lg mb-4">No items yet</p>
                <p className="text-gray-500 mb-6">Start by adding your first password or note</p>
                <div className="flex justify-center space-x-4">
                  <button
                    onClick={() => handleAddItem('password')}
                    className="btn btn-primary"
                  >
                    Add Password
                  </button>
                  <button
                    onClick={() => handleAddItem('note')}
                    className="btn btn-secondary"
                  >
                    Add Note
                  </button>
                </div>
              </div>
            ) : (
              <>
                <div className="flex justify-between items-center mb-4">
                  <h2 className="text-lg font-medium">
                    Items ({items.length})
                  </h2>
                  <div className="flex space-x-2">
                    <button
                      onClick={handleRefresh}
                      className="btn btn-secondary px-3 py-1 text-sm"
                    >
                      Refresh
                    </button>
                    <button
                      onClick={() => handleAddItem('password')}
                      className="btn btn-primary px-3 py-1 text-sm"
                    >
                      + Password
                    </button>
                    <button
                      onClick={() => handleAddItem('note')}
                      className="btn btn-primary px-3 py-1 text-sm"
                    >
                      + Note
                    </button>
                  </div>
                </div>
                
                <ItemList
                  items={items}
                  onRefresh={handleRefresh}
                />
              </>
            )}
          </>
        )}
      </main>

      {/* Modals */}
      {showAddModal && (
        <AddItemModal
          onClose={() => setShowAddModal(false)}
          onAdded={handleItemAdded}
        />
      )}

      {showSettings && (
        <SettingsModal
          onClose={() => setShowSettings(false)}
          vaultId={vaultId}
          vaultName={vaultName}
        />
      )}
    </div>
  );
}

export default UnlockedScreen;
