import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { Settings as SettingsIcon, Save, RefreshCw, FolderOpen } from "lucide-react";

interface Config {
  source_dir: string;
  drives: DriveConfig[];
  file_categories: Record<string, string[]>;
}

interface DriveConfig {
  label: string;
  category: string;
  mount_point: string | null;
}

export default function ConfigEditor() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [errors, setErrors] = useState<string[]>([]);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await invoke<Config>("get_config");
      setConfig(data);
    } catch (error) {
      console.error("Failed to load config:", error);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    if (!config) return;

    setSaving(true);
    setErrors([]);

    try {
      // Validate first
      const validationErrors = await invoke<string[]>("validate_config_cmd");
      if (validationErrors.length > 0) {
        setErrors(validationErrors);
        setSaving(false);
        return;
      }

      await invoke("update_config", { config });
      alert("Configuration saved successfully!");
    } catch (error) {
      console.error("Failed to save config:", error);
      alert("Failed to save configuration");
    } finally {
      setSaving(false);
    }
  };

  const selectSourceDir = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (selected && typeof selected === "string" && config) {
      setConfig({ ...config, source_dir: selected });
    }
  };

  const addExtension = (category: string) => {
    if (!config) return;
    const ext = prompt(`Add file extension for ${category}:`);
    if (!ext) return;

    const newConfig = { ...config };
    if (!newConfig.file_categories[category]) {
      newConfig.file_categories[category] = [];
    }
    newConfig.file_categories[category].push(ext.replace(".", ""));
    setConfig(newConfig);
  };

  const removeExtension = (category: string, ext: string) => {
    if (!config) return;

    const newConfig = { ...config };
    newConfig.file_categories[category] = newConfig.file_categories[category].filter(
      (e) => e !== ext
    );
    setConfig(newConfig);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="text-center py-16">
        <SettingsIcon size={64} className="mx-auto mb-4 text-gray-600" />
        <h3 className="text-xl font-semibold mb-2">Configuration not found</h3>
        <p className="text-gray-400">Unable to load configuration</p>
      </div>
    );
  }

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2">Configuration</h1>
          <p className="text-gray-400">
            Manage application settings and file categories
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={loadConfig}
            className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            <RefreshCw size={20} />
            Reset
          </button>
          <button
            onClick={saveConfig}
            disabled={saving}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors disabled:opacity-50"
          >
            <Save size={20} />
            {saving ? "Saving..." : "Save Changes"}
          </button>
        </div>
      </div>

      {errors.length > 0 && (
        <div className="bg-red-500/20 border border-red-500 rounded-lg p-4 mb-6">
          <h3 className="font-semibold mb-2">Validation Errors:</h3>
          <ul className="list-disc list-inside space-y-1">
            {errors.map((error, i) => (
              <li key={i} className="text-sm">{error}</li>
            ))}
          </ul>
        </div>
      )}

      <div className="space-y-6">
        {/* Source Directory */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h2 className="text-xl font-semibold mb-4">Source Directory</h2>
          <div className="flex gap-3">
            <input
              type="text"
              value={config.source_dir}
              onChange={(e) => setConfig({ ...config, source_dir: e.target.value })}
              className="flex-1 px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              placeholder="/path/to/source"
            />
            <button
              onClick={selectSourceDir}
              className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
            >
              <FolderOpen size={20} />
              Browse
            </button>
          </div>
          <p className="text-sm text-gray-400 mt-2">
            The directory to watch for new files
          </p>
        </div>

        {/* File Categories */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h2 className="text-xl font-semibold mb-4">File Categories</h2>
          <p className="text-sm text-gray-400 mb-4">
            Define which file extensions belong to each category
          </p>

          <div className="space-y-4">
            {Object.entries(config.file_categories).map(([category, extensions]) => (
              <div key={category} className="border border-gray-700 rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="font-semibold capitalize">{category}</h3>
                  <button
                    onClick={() => addExtension(category)}
                    className="text-sm px-3 py-1 bg-primary-600 hover:bg-primary-700 rounded transition-colors"
                  >
                    Add Extension
                  </button>
                </div>

                <div className="flex flex-wrap gap-2">
                  {extensions.map((ext) => (
                    <div
                      key={ext}
                      className="flex items-center gap-2 px-3 py-1 bg-gray-700 rounded-full"
                    >
                      <span className="text-sm">.{ext}</span>
                      <button
                        onClick={() => removeExtension(category, ext)}
                        className="text-gray-400 hover:text-red-400 transition-colors"
                      >
                        ×
                      </button>
                    </div>
                  ))}
                  {extensions.length === 0 && (
                    <span className="text-sm text-gray-500 italic">
                      No extensions defined
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Registered Drives Summary */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h2 className="text-xl font-semibold mb-4">Registered Drives</h2>
          <p className="text-sm text-gray-400 mb-4">
            Manage drives in the Drive Manager section
          </p>

          {config.drives.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              No drives registered yet
            </div>
          ) : (
            <div className="space-y-2">
              {config.drives.map((drive) => (
                <div
                  key={drive.label}
                  className="flex items-center justify-between p-3 bg-gray-700 rounded-lg"
                >
                  <div>
                    <div className="font-semibold">{drive.label}</div>
                    <div className="text-sm text-gray-400 uppercase">
                      {drive.category}
                    </div>
                  </div>
                  {drive.mount_point && (
                    <div className="text-sm text-gray-400 font-mono">
                      {drive.mount_point}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
