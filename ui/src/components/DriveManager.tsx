import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import {
  HardDrive,
  Plus,
  Trash2,
  RefreshCw,
  CheckCircle,
  XCircle,
} from "lucide-react";

interface DriveInfo {
  label: string;
  category: string;
  mount_point: string | null;
  is_connected: boolean;
  total_space: number | null;
  available_space: number | null;
}

export default function DriveManager() {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);

  useEffect(() => {
    loadDrives();
    const interval = setInterval(loadDrives, 10000);
    return () => clearInterval(interval);
  }, []);

  const loadDrives = async () => {
    try {
      const data = await invoke<DriveInfo[]>("get_drives");
      setDrives(data);
    } catch (error) {
      console.error("Failed to load drives:", error);
    } finally {
      setLoading(false);
    }
  };

  const removeDrive = async (label: string) => {
    if (!confirm(`Remove drive "${label}"?`)) return;

    try {
      await invoke("remove_drive", { label });
      await loadDrives();
    } catch (error) {
      console.error("Failed to remove drive:", error);
      alert("Failed to remove drive");
    }
  };

  const formatBytes = (bytes: number | null) => {
    if (!bytes) return "N/A";
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(2)} GB`;
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
          <h1 className="text-3xl font-bold mb-2">Drive Manager</h1>
          <p className="text-gray-400">
            Manage registered USB drives and monitor their status
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={loadDrives}
            className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            <RefreshCw size={20} />
            Refresh
          </button>
          <button
            onClick={() => setShowAddModal(true)}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
          >
            <Plus size={20} />
            Add Drive
          </button>
        </div>
      </div>

      {drives.length === 0 ? (
        <div className="text-center py-16">
          <HardDrive size={64} className="mx-auto mb-4 text-gray-600" />
          <h3 className="text-xl font-semibold mb-2">No drives registered</h3>
          <p className="text-gray-400 mb-6">
            Add your first USB drive to start syncing files
          </p>
          <button
            onClick={() => setShowAddModal(true)}
            className="px-6 py-3 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
          >
            Register First Drive
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {drives.map((drive) => (
            <DriveCard
              key={drive.label}
              drive={drive}
              onRemove={removeDrive}
              formatBytes={formatBytes}
            />
          ))}
        </div>
      )}

      {showAddModal && (
        <AddDriveModal
          onClose={() => setShowAddModal(false)}
          onSuccess={() => {
            setShowAddModal(false);
            loadDrives();
          }}
        />
      )}
    </div>
  );
}

interface DriveCardProps {
  drive: DriveInfo;
  onRemove: (label: string) => void;
  formatBytes: (bytes: number | null) => string;
}

function DriveCard({ drive, onRemove, formatBytes }: DriveCardProps) {
  const usedSpace = drive.total_space && drive.available_space
    ? drive.total_space - drive.available_space
    : null;
  
  const usagePercent = drive.total_space && usedSpace
    ? (usedSpace / drive.total_space) * 100
    : 0;

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700 relative">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="p-3 bg-gray-700 rounded-lg">
            <HardDrive size={24} className={drive.is_connected ? "text-green-500" : "text-gray-500"} />
          </div>
          <div>
            <h3 className="text-lg font-semibold">{drive.label}</h3>
            <span className="text-sm text-gray-400 uppercase">{drive.category}</span>
          </div>
        </div>
        <button
          onClick={() => onRemove(drive.label)}
          className="text-gray-400 hover:text-red-500 transition-colors"
        >
          <Trash2 size={18} />
        </button>
      </div>

      <div className="space-y-3">
        <div className="flex items-center gap-2">
          {drive.is_connected ? (
            <>
              <CheckCircle size={16} className="text-green-500" />
              <span className="text-sm text-green-400">Connected</span>
            </>
          ) : (
            <>
              <XCircle size={16} className="text-gray-500" />
              <span className="text-sm text-gray-400">Offline</span>
            </>
          )}
        </div>

        {drive.mount_point && (
          <div className="text-sm">
            <span className="text-gray-400">Mount: </span>
            <span className="text-gray-300 font-mono text-xs">{drive.mount_point}</span>
          </div>
        )}

        {drive.is_connected && drive.total_space && (
          <>
            <div className="pt-2">
              <div className="flex justify-between text-sm mb-1">
                <span className="text-gray-400">Storage</span>
                <span className="text-gray-300">
                  {formatBytes(usedSpace)} / {formatBytes(drive.total_space)}
                </span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div
                  className="bg-primary-500 h-2 rounded-full transition-all"
                  style={{ width: `${usagePercent}%` }}
                ></div>
              </div>
              <div className="text-xs text-gray-400 mt-1">
                {formatBytes(drive.available_space)} available
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

interface AddDriveModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

function AddDriveModal({ onClose, onSuccess }: AddDriveModalProps) {
  const [label, setLabel] = useState("");
  const [category, setCategory] = useState("documents");
  const [path, setPath] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!label) return;

    setSubmitting(true);
    try {
      await invoke("register_drive_cmd", {
        label,
        category,
        path: path || null,
      });
      onSuccess();
    } catch (error) {
      console.error("Failed to register drive:", error);
      alert("Failed to register drive");
    } finally {
      setSubmitting(false);
    }
  };

  const selectPath = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (selected && typeof selected === "string") {
      setPath(selected);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md border border-gray-700">
        <h2 className="text-2xl font-bold mb-4">Register New Drive</h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">Drive Label *</label>
            <input
              type="text"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              placeholder="e.g., USB_BACKUP_01"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">Category *</label>
            <select
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              className="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="documents">Documents</option>
              <option value="images">Images</option>
              <option value="videos">Videos</option>
              <option value="audio">Audio</option>
              <option value="archives">Archives</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">
              Mount Point (Optional)
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                className="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
                placeholder="/media/usb"
              />
              <button
                type="button"
                onClick={selectPath}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
              >
                Browse
              </button>
            </div>
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50"
            >
              {submitting ? "Registering..." : "Register"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
