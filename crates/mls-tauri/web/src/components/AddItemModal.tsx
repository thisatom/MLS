/**
 * Add Item Modal Component
 */

import { useState } from 'react';
import { commands } from '../commands';

interface AddItemModalProps {
  onClose: () => void;
  onAdded: () => void;
}

function AddItemModal({ onClose, onAdded }: AddItemModalProps) {
  const [itemType, setItemType] = useState<'password' | 'note' | 'generic'>('password');
  const [name, setName] = useState('');
  const [service, setService] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [url, setUrl] = useState('');
  const [notes, setNotes] = useState('');
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name) {
      setError('Name is required');
      return;
    }

    if (itemType === 'password' && !password) {
      setError('Password is required');
      return;
    }

    if (itemType === 'note' && !content) {
      setError('Content is required');
      return;
    }

    setLoading(true);

    try {
      await commands.addItem({
        type: itemType,
        name,
        service: itemType === 'password' ? service : undefined,
        username: itemType === 'password' ? username : undefined,
        password: itemType === 'password' ? password : undefined,
        url: itemType === 'password' ? url : undefined,
        content: itemType === 'note' ? content : undefined,
        notes: itemType === 'password' ? notes : undefined,
      });

      onAdded();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add item');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay">
      <div className="modal">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold">Add New Item</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
          >
            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {error && (
          <div className="bg-red-600 text-white px-4 py-2 rounded-md mb-4">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Item Type Selector */}
          <div className="flex space-x-2 mb-4">
            <button
              type="button"
              onClick={() => setItemType('password')}
              className={`btn flex-1 ${itemType === 'password' ? 'btn-primary' : 'btn-secondary'}`}
            >
              Password
            </button>
            <button
              type="button"
              onClick={() => setItemType('note')}
              className={`btn flex-1 ${itemType === 'note' ? 'btn-primary' : 'btn-secondary'}`}
            >
              Note
            </button>
            <button
              type="button"
              onClick={() => setItemType('generic')}
              className={`btn flex-1 ${itemType === 'generic' ? 'btn-primary' : 'btn-secondary'}`}
            >
              Generic
            </button>
          </div>

          {/* Common Fields */}
          <div>
            <label htmlFor="name" className="block text-sm font-medium text-gray-300 mb-1">
              Name
            </label>
            <input
              type="text"
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="input"
              placeholder="Item name"
              required
            />
          </div>

          {/* Password Type Fields */}
          {itemType === 'password' && (
            <>
              <div>
                <label htmlFor="service" className="block text-sm font-medium text-gray-300 mb-1">
                  Service / Website
                </label>
                <input
                  type="text"
                  id="service"
                  value={service}
                  onChange={(e) => setService(e.target.value)}
                  className="input"
                  placeholder="example.com"
                />
              </div>
              
              <div>
                <label htmlFor="username" className="block text-sm font-medium text-gray-300 mb-1">
                  Username
                </label>
                <input
                  type="text"
                  id="username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="input"
                  placeholder="your username"
                />
              </div>
              
              <div>
                <label htmlFor="password" className="block text-sm font-medium text-gray-300 mb-1">
                  Password
                </label>
                <input
                  type="password"
                  id="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="input"
                  placeholder="your password"
                  required
                />
              </div>
              
              <div>
                <label htmlFor="url" className="block text-sm font-medium text-gray-300 mb-1">
                  URL
                </label>
                <input
                  type="url"
                  id="url"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  className="input"
                  placeholder="https://example.com"
                />
              </div>
              
              <div>
                <label htmlFor="notes" className="block text-sm font-medium text-gray-300 mb-1">
                  Notes
                </label>
                <textarea
                  id="notes"
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                  className="input min-h-[100px] resize-none"
                  placeholder="Additional notes"
                />
              </div>
            </>
          )}

          {/* Note Type Fields */}
          {itemType === 'note' && (
            <div>
              <label htmlFor="content" className="block text-sm font-medium text-gray-300 mb-1">
                Content
              </label>
              <textarea
                id="content"
                value={content}
                onChange={(e) => setContent(e.target.value)}
                className="input min-h-[150px] resize-none"
                placeholder="Your note content"
                required
              />
            </div>
          )}

          {/* Generic Type Fields */}
          {itemType === 'generic' && (
            <div>
              <label htmlFor="data" className="block text-sm font-medium text-gray-300 mb-1">
                Data
              </label>
              <textarea
                id="data"
                value={content}
                onChange={(e) => setContent(e.target.value)}
                className="input min-h-[100px] resize-none"
                placeholder="Generic data"
              />
            </div>
          )}

          <div className="flex justify-end space-x-2 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="btn btn-secondary"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="btn btn-primary"
            >
              {loading ? (
                <span className="flex items-center">
                  <span className="animate-spin rounded-full h-4 w-4 border-t-2 border-b-2 border-white mr-2"></span>
                  Adding...
                </span>
              ) : (
                'Add Item'
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default AddItemModal;
