import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Clock, FileText, RefreshCw, Trash2, PlayCircle } from "lucide-react";

interface PendingFileInfo {
  file_path: string;
  file_name: string;
  file_type: string;
  category: string;
  target_drive: string;
  size: number;
  added_at: string;
}

export default function PendingQueue() {
  const [files, setFiles] = useState<PendingFileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);

  useEffect(() => {
    loadPendingFiles();
    const interval = setInterval(loadPendingFiles, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadPendingFiles = async () => {
    try {
      const data = await invoke<PendingFileInfo[]>("get_pending_files");
      setFiles(data);
    } catch (error) {
      console.error("Failed to load pending files:", error);
    } finally {
      setLoading(false);
    }
  };

  const syncAll = async () => {
    setSyncing(true);
    try {
      const count = await invoke<number>("sync_pending_cmd");
      alert(`Successfully synced ${count} files`);
      await loadPendingFiles();
    } catch (error) {
      console.error("Failed to sync pending files:", error);
      alert("Failed to sync pending files");
    } finally {
      setSyncing(false);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
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
          <h1 className="text-3xl font-bold mb-2">Pending Queue</h1>
          <p className="text-gray-400">
            Files waiting for target drives to be connected
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={loadPendingFiles}
            className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            <RefreshCw size={20} />
            Refresh
          </button>
          {files.length > 0 && (
            <button
              onClick={syncAll}
              disabled={syncing}
              className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50"
            >
              <PlayCircle size={20} className={syncing ? "animate-spin" : ""} />
              {syncing ? "Syncing..." : "Sync All"}
            </button>
          )}
        </div>
      </div>

      {files.length === 0 ? (
        <div className="text-center py-16">
          <Clock size={64} className="mx-auto mb-4 text-gray-600" />
          <h3 className="text-xl font-semibold mb-2">No pending files</h3>
          <p className="text-gray-400">
            All files have been synced or no files are waiting for drives
          </p>
        </div>
      ) : (
        <>
          <div className="bg-gray-800 rounded-lg p-4 mb-6 border border-gray-700">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="text-2xl font-bold text-primary-500">
                  {files.length}
                </div>
                <div className="text-sm text-gray-400">
                  files pending sync
                </div>
              </div>
              <div className="text-sm text-gray-400">
                Total size: {formatBytes(files.reduce((acc, f) => acc + f.size, 0))}
              </div>
            </div>
          </div>

          <div className="space-y-3">
            {files.map((file, index) => (
              <PendingFileCard
                key={index}
                file={file}
                formatBytes={formatBytes}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
}

interface PendingFileCardProps {
  file: PendingFileInfo;
  formatBytes: (bytes: number) => string;
}

function PendingFileCard({ file, formatBytes }: PendingFileCardProps) {
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

  const getCategoryBg = (category: string) => {
    const colors: Record<string, string> = {
      documents: "bg-blue-500/20",
      images: "bg-green-500/20",
      videos: "bg-purple-500/20",
      audio: "bg-yellow-500/20",
      archives: "bg-red-500/20",
    };
    return colors[category] || "bg-gray-500/20";
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4 border border-gray-700 hover:border-gray-600 transition-colors">
      <div className="flex items-center gap-4">
        <div className={`p-3 rounded-lg ${getCategoryBg(file.category)}`}>
          <FileText className={getCategoryColor(file.category)} size={24} />
        </div>

        <div className="flex-1 min-w-0">
          <h3 className="font-semibold truncate">{file.file_name}</h3>
          <div className="flex items-center gap-4 mt-1 text-sm text-gray-400">
            <span className="uppercase">{file.file_type}</span>
            <span>•</span>
            <span>{formatBytes(file.size)}</span>
            <span>•</span>
            <span className={getCategoryColor(file.category)}>
              {file.category}
            </span>
          </div>
        </div>

        <div className="text-right">
          <div className="text-sm font-medium text-gray-300">
            → {file.target_drive}
          </div>
          <div className="text-xs text-gray-500 mt-1">
            Waiting for drive
          </div>
        </div>
      </div>
    </div>
  );
}
