/**
 * Item List Component
 */

import { useState } from 'react';
import { commands, Item } from '../commands';

interface ItemListProps {
  items: Item[];
  onRefresh: () => void;
}

function ItemList({ items, onRefresh }: ItemListProps) {
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const handleDelete = async (itemId: string) => {
    if (!confirm('Are you sure you want to delete this item?')) {
      return;
    }

    setDeletingId(itemId);
    
    try {
      await commands.removeItem({ itemId, force: true });
      onRefresh();
    } catch (err) {
      console.error('Failed to delete item:', err);
    } finally {
      setDeletingId(null);
    }
  };

  const getItemIcon = (type: string) => {
    switch (type) {
      case 'password':
        return (
          <svg className="w-5 h-5 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        );
      case 'note':
        return (
          <svg className="w-5 h-5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        );
      default:
        return (
          <svg className="w-5 h-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        );
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const getItemTitle = (item: Item) => {
    if ('service' in item) {
      return item.service;
    }
    if ('title' in item) {
      return item.title || item.name;
    }
    return item.name;
  };

  const getItemSubtitle = (item: Item) => {
    if ('service' in item) {
      return item.username ? `Username: ${item.username}` : item.name;
    }
    if ('title' in item) {
      return item.name;
    }
    return item.name;
  };

  if (items.length === 0) {
    return null;
  }

  return (
    <div className="space-y-2">
      {items.map((item) => (
        <div
          key={item.id}
          className="card flex items-center justify-between hover:bg-gray-700 transition-colors"
        >
          <div className="flex items-center space-x-4 flex-1">
            <div className="flex-shrink-0">
              {getItemIcon(item.type)}
            </div>
            <div className="flex-1 min-w-0">
              <h3 className="font-medium truncate">{getItemTitle(item)}</h3>
              <p className="text-sm text-gray-400 truncate">{getItemSubtitle(item)}</p>
            </div>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-500">{formatDate(item.updatedAt)}</span>
              <button
                onClick={() => handleDelete(item.id)}
                disabled={deletingId === item.id}
                className="text-gray-400 hover:text-red-400 transition-colors disabled:opacity-50"
              >
                {deletingId === item.id ? (
                  <span className="animate-spin rounded-full h-4 w-4 border-t-2 border-b-2 border-red-500"></span>
                ) : (
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                )}
              </button>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

export default ItemList;
