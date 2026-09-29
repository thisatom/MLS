/**
 * Settings Modal Component
 */

import { commands } from '../commands';

interface SettingsModalProps {
  onClose: () => void;
  vaultId: string;
  vaultName: string;
}

function SettingsModal({ onClose, vaultId, vaultName }: SettingsModalProps) {
  const [syncStatus, setSyncStatus] = useState<{
    isSyncing: boolean;
    lastSyncTime?: string;
    connectedDevices: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);

  const loadSyncStatus = async () => {
    setLoading(true);
    try {
      const result = await commands.getSyncStatus();
      setSyncStatus(result);
    } catch (err) {
      console.error('Failed to load sync status:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSyncStatus();
  }, []);

  const handleLock = async () => {
    try {
      await commands.lock();
      onClose();
    } catch (err) {
      console.error('Failed to lock:', err);
    }
  };

  return (
    <div className="modal-overlay">
      <div className="modal max-w-lg">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold">Settings</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
          >
            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="space-y-6">
          {/* Vault Information */}
          <section>
            <h3 className="text-lg font-medium mb-3">Vault Information</h3>
            <div className="space-y-2">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Name</label>
                <div className="bg-gray-700 px-3 py-2 rounded-md">{vaultName}</div>
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Vault ID</label>
                <div className="bg-gray-700 px-3 py-2 rounded-md font-mono text-sm">
                  {vaultId}
                </div>
              </div>
            </div>
          </section>

          {/* Synchronization */}
          <section>
            <h3 className="text-lg font-medium mb-3">Synchronization</h3>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-300">Status</span>
                <span className={`text-sm ${syncStatus?.isSyncing ? 'text-green-400' : 'text-gray-400'}`}>
                  {syncStatus?.isSyncing ? 'Syncing...' : 'Idle'}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-300">Connected Devices</span>
                <span className="text-sm text-gray-400">{syncStatus?.connectedDevices || 0}</span>
              </div>
              {syncStatus?.lastSyncTime && (
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-300">Last Sync</span>
                  <span className="text-sm text-gray-400">{new Date(syncStatus.lastSyncTime).toLocaleString()}</span>
                </div>
              )}
            </div>
          </section>

          {/* Danger Zone */}
          <section>
            <h3 className="text-lg font-medium mb-3 text-red-400">Danger Zone</h3>
            <button
              onClick={handleLock}
              className="btn btn-danger w-full"
            >
              Lock Vault
            </button>
          </section>
        </div>

        <div className="flex justify-end pt-4">
          <button onClick={onClose} className="btn btn-secondary">
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

export default SettingsModal;
