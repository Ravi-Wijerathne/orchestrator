import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { History as HistoryIcon, RefreshCw, Trash2, FileText, CheckCircle } from "lucide-react";
import { format } from "date-fns";

interface SyncHistoryEntry {
  id: string;
  source_path: string;
  file_name: string;
  target_drive: string;
  target_path: string;
  file_type: string;
  category: string;
  file_hash: string;
  synced_at: string;
  status: string;
}

export default function SyncHistory() {
  const [history, setHistory] = useState<SyncHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [limit, setLimit] = useState(100);

  useEffect(() => {
    loadHistory();
  }, [limit]);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const data = await invoke<SyncHistoryEntry[]>("get_sync_history", { limit });
      setHistory(data);
    } catch (error) {
      console.error("Failed to load history:", error);
    } finally {
      setLoading(false);
    }
  };

  const clearHistory = async () => {
    if (!confirm("Are you sure you want to clear all sync history? This cannot be undone.")) {
      return;
    }

    try {
      await invoke("clear_history");
      await loadHistory();
    } catch (error) {
      console.error("Failed to clear history:", error);
      alert("Failed to clear history");
    }
  };

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return format(date, "MMM dd, yyyy HH:mm:ss");
    } catch {
      return dateStr;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
      </div>
    );
  }

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2">Sync History</h1>
          <p className="text-gray-400">
            Complete timeline of all file synchronization operations
          </p>
        </div>
        <div className="flex gap-3">
          <select
            value={limit}
            onChange={(e) => setLimit(Number(e.target.value))}
            className="px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value={50}>Last 50</option>
            <option value={100}>Last 100</option>
            <option value={500}>Last 500</option>
            <option value={1000}>Last 1000</option>
          </select>
          <button
            onClick={loadHistory}
            className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            <RefreshCw size={20} />
            Refresh
          </button>
          <button
            onClick={clearHistory}
            className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
          >
            <Trash2 size={20} />
            Clear History
          </button>
        </div>
      </div>

      {history.length === 0 ? (
        <div className="text-center py-16">
          <HistoryIcon size={64} className="mx-auto mb-4 text-gray-600" />
          <h3 className="text-xl font-semibold mb-2">No sync history</h3>
          <p className="text-gray-400">
            Sync history will appear here as files are synchronized
          </p>
        </div>
      ) : (
        <>
          <div className="bg-gray-800 rounded-lg p-4 mb-6 border border-gray-700">
            <div className="text-sm text-gray-400">
              Showing {history.length} sync operations
            </div>
          </div>

          <div className="space-y-3">
            {history.map((entry) => (
              <HistoryCard key={entry.id} entry={entry} formatDate={formatDate} />
            ))}
          </div>
        </>
      )}
    </div>
  );
}

interface HistoryCardProps {
  entry: SyncHistoryEntry;
  formatDate: (date: string) => string;
}

function HistoryCard({ entry, formatDate }: HistoryCardProps) {
  const getCategoryColor = (category: string) => {
    const colors: Record<string, string> = {
      documents: "text-blue-400",
      images: "text-green-400",
      videos: "text-purple-400",
      audio: "text-yellow-400",
      archives: "text-red-400",
    };
    return colors[category] || "text-gray-400";
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
      <div className="flex items-start gap-4">
        <div className="p-2 bg-green-500/20 rounded-lg">
          <CheckCircle className="text-green-400" size={20} />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 mb-2">
            <FileText size={16} className={getCategoryColor(entry.category)} />
            <h3 className="font-semibold truncate">{entry.file_name}</h3>
            <span className="text-xs uppercase text-gray-500">{entry.file_type}</span>
          </div>

          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-400">Source:</span>
              <div className="text-gray-300 truncate font-mono text-xs mt-1">
                {entry.source_path}
              </div>
            </div>
            <div>
              <span className="text-gray-400">Target:</span>
              <div className="text-gray-300 truncate font-mono text-xs mt-1">
                {entry.target_drive}: {entry.target_path}
              </div>
            </div>
          </div>

          <div className="flex items-center gap-4 mt-3 text-xs text-gray-500">
            <span className={getCategoryColor(entry.category)}>
              {entry.category}
            </span>
            <span>•</span>
            <span>{formatDate(entry.synced_at)}</span>
            <span>•</span>
            <span className="font-mono">Hash: {entry.file_hash.substring(0, 16)}...</span>
          </div>
        </div>
      </div>
    </div>
  );
}
