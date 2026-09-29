/**
 * MLS - Main Application Component
 * Handles routing and layout for the entire application
 */

import { useState, useEffect } from 'react';
import { SessionStatus } from './types';
import { commands } from './commands';
import LockedScreen from './screens/LockedScreen';
import UnlockedScreen from './screens/UnlockedScreen';
import InitScreen from './screens/InitScreen';

function App() {
  const [sessionStatus, setSessionStatus] = useState<SessionStatus>('initializing');
  const [vaultId, setVaultId] = useState<string | null>(null);
  const [vaultName, setVaultName] = useState<string | null>(null);
  const [hasVault, setHasVault] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(true);

  // Check session status on mount
  useEffect(() => {
    const checkStatus = async () => {
      try {
        const result = await commands.getSessionStatus();
        setSessionStatus(result.status);
        setVaultId(result.vaultId || null);
        setVaultName(result.vaultName || null);
        
        // Check if vault exists
        const vaults = await commands.listVaults();
        setHasVault(vaults.vaults.length > 0);
      } catch (error) {
        console.error('Failed to check session status:', error);
      } finally {
        setLoading(false);
      }
    };
    
    checkStatus();
    
    // Set up periodic status check
    const interval = setInterval(checkStatus, 30000);
    return () => clearInterval(interval);
  }, []);

  // Handle successful unlock
  const handleUnlock = (vaultId: string, vaultName: string) => {
    setSessionStatus('unlocked');
    setVaultId(vaultId);
    setVaultName(vaultName);
  };

  // Handle successful init
  const handleInit = (vaultId: string, vaultName: string) => {
    setHasVault(true);
    setSessionStatus('unlocked');
    setVaultId(vaultId);
    setVaultName(vaultName);
  };

  // Handle lock
  const handleLock = () => {
    setSessionStatus('locked');
    setVaultId(null);
    setVaultName(null);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  // Show init screen if no vault exists
  if (!hasVault) {
    return <InitScreen onInit={handleInit} />;
  }

  // Show locked screen if vault exists but not unlocked
  if (sessionStatus === 'locked') {
    return <LockedScreen onUnlock={handleUnlock} vaultId={vaultId} vaultName={vaultName} />;
  }

  // Show main app if unlocked
  return (
    <UnlockedScreen
      vaultId={vaultId!}
      vaultName={vaultName!}
      onLock={handleLock}
    />
  );
}

export default App;
