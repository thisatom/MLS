/**
 * Locked Screen - Unlock existing vault
 */

import { useState } from 'react';
import { commands } from '../commands';

interface LockedScreenProps {
  onUnlock: (vaultId: string, vaultName: string) => void;
  vaultId?: string | null;
  vaultName?: string | null;
}

function LockedScreen({ onUnlock, vaultId, vaultName }: LockedScreenProps) {
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    
    if (!password) {
      setError('Password is required');
      return;
    }

    setLoading(true);
    
    try {
      const result = await commands.unlock({
        vaultId: vaultId || undefined,
        masterPassword: password,
      });
      
      onUnlock(result.vaultId, result.vaultName);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to unlock vault');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-900 p-4">
      <div className="w-full max-w-md">
        <div className="card">
          <h1 className="text-2xl font-bold text-center mb-2">
            MLS
          </h1>
          
          <p className="text-gray-400 text-center mb-1">
            {vaultName || 'My Life Storage'}
          </p>
          
          {vaultId && (
            <p className="text-gray-500 text-center text-sm mb-6">
              Vault ID: {vaultId.slice(0, 8)}...
            </p>
          )}
          
          {error && (
            <div className="bg-red-600 text-white px-4 py-2 rounded-md mb-4">
              {error}
            </div>
          )}
          
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="password" className="block text-sm font-medium text-gray-300 mb-1">
                Master Password
              </label>
              <input
                type="password"
                id="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="input"
                placeholder="Enter master password"
                autoFocus
                required
              />
            </div>
            
            <button
              type="submit"
              disabled={loading}
              className="btn btn-primary w-full"
            >
              {loading ? (
                <span className="flex items-center justify-center">
                  <span className="animate-spin rounded-full h-4 w-4 border-t-2 border-b-2 border-white mr-2"></span>
                  Unlocking...
                </span>
              ) : (
                'Unlock Vault'
              )}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}

export default LockedScreen;
