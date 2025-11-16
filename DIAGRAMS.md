# 📊 Visual System Diagram

## 🎨 Complete System Overview

```mermaid
graph TB
    subgraph CLI["USER INTERFACE (CLI)"]
        CMD1[init]
        CMD2[register-drive]
        CMD3[run]
        CMD4[status]
        CMD5[sync-once]
    end
    
    subgraph APP["APPLICATION LAYER"]
        HANDLERS[Command Handlers<br/>• Initialize config<br/>• Register drives<br/>• Execute sync<br/>• Show statistics]
    end
    
    subgraph CORE["CORE ENGINE (Domain Layer)"]
        CONFIG[Config Manager]
        CLASSIFIER[File Classifier]
        STATE[State Manager]
        
        SYNC["SYNC MANAGER<br/>────────────────<br/>• Check if synced (hash)<br/>• Find target drive<br/>• Copy if online<br/>• Queue if offline<br/>• Resume on reconnect"]
        
        WATCHER[File Watcher]
        DETECTOR[Drive Detector]
    end
    
    subgraph INFRA["INFRASTRUCTURE LAYER"]
        FS[File I/O<br/>tokio::fs]
        DB[Database<br/>sled]
        SYS[System Info<br/>sysinfo]
    end
    
    CLI --> APP
    APP --> CORE
    CONFIG --> SYNC
    CLASSIFIER --> SYNC
    STATE --> SYNC
    SYNC --> WATCHER
    SYNC --> DETECTOR
    CORE --> INFRA
    
    style CLI fill:#e3f2fd
    style APP fill:#fff9c4
    style CORE fill:#c8e6c9
    style INFRA fill:#ffccbc
    style SYNC fill:#b2dfdb
```

## 🔄 Data Flow Diagram

```mermaid
flowchart TD
    USER[USER] -->|commands| CLI[CLI - clap parser]
    
    CLI --> HANDLER[Command Handler]
    CLI --> CONFIG[Config Manager]
    
    CONFIG -->|reads/writes| TOML[config.toml]
    
    HANDLER --> SYNC[Sync Manager]
    
    subgraph SYNC_PROCESS["Sync Manager Process"]
        S1[1. Get file from source]
        S2[2. Classify file type]
        S3[3. Calculate hash]
        S4[4. Check if synced]
        S5[5. Find target drive]
        S6[6. Copy or queue]
        S1 --> S2 --> S3 --> S4 --> S5 --> S6
    end
    
    SYNC --> SYNC_PROCESS
    SYNC_PROCESS --> STATE[State Manager<br/>sled DB]
    SYNC_PROCESS --> DETECTOR[Drive Detector<br/>sysinfo]
    
    STATE --> DB[(orchestrator.db)]
    DETECTOR --> DRIVES[USB Drives<br/>E:\ F:\ G:\]
    
    style USER fill:#e3f2fd
    style CLI fill:#fff9c4
    style SYNC fill:#c8e6c9
    style STATE fill:#ffccbc
    style DETECTOR fill:#f8bbd0
```

## 🎬 Sync Process Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     SYNC PROCESS                                 │
└─────────────────────────────────────────────────────────────────┘

  START
    │
    ▼
┌───────────────────┐
│ File Detected     │  ◄── File Watcher (notify)
│  photo.jpg        │      monitors HDD directory
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Classify Type     │  ◄── infer crate reads
│  → Image          │      magic bytes (MIME)
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Calculate Hash    │  ◄── BLAKE3 hashing
│  → abc123...      │      (fast & secure)
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Already Synced?   │  ◄── Check sled database
└─────────┬─────────┘      for existing hash
          │
    ┌─────┴─────┐
    │           │
   YES         NO
    │           │
    ▼           ▼
┌────────┐  ┌──────────────────┐
│  SKIP  │  │ Find Target USB  │  ◄── Config lookup
└────────┘  │  → ImageUSB      │      images → USB1
            └─────────┬────────┘
                      │
                      ▼
            ┌──────────────────┐
            │ USB Connected?   │  ◄── Drive detector
            └─────────┬────────┘      checks sysinfo
                      │
                ┌─────┴─────┐
                │           │
               YES         NO
                │           │
                ▼           ▼
     ┌────────────────┐  ┌──────────────┐
     │  COPY FILE     │  │ ADD TO QUEUE │
     │  HDD → USB     │  │  (pending)   │
     └───────┬────────┘  └──────┬───────┘
             │                  │
             ▼                  │
     ┌────────────────┐         │
     │ Update State   │         │
     │ Save hash/path │         │
     └───────┬────────┘         │
             │                  │
             ▼                  │
          ┌──────┐              │
          │ DONE │              │
          └──────┘              │
                                │
                                ▼
                      ┌──────────────────┐
                      │ Wait for USB     │
                      │ reconnection...  │
                      └────────┬─────────┘
                               │
                               │ USB plugged in!
                               ▼
                      ┌──────────────────┐
                      │ Process Queue    │
                      │ Sync all pending │
                      └────────┬─────────┘
                               │
                               ▼
                            ┌──────┐
                            │ DONE │
                            └──────┘
```

## 🗄️ State Database Schema

```mermaid
erDiagram
    FILE_STATE {
        string key "file:source_path"
        string source_path
        string hash "BLAKE3"
        int size
        int last_synced "timestamp"
        string target_drive "UUID"
        string target_path
        string file_category
    }
    
    PENDING_SYNC {
        string key "pending:source_path"
        string source_path
        string file_category
        string target_drive "UUID"
        string hash "BLAKE3"
        int size
        int created_at "timestamp"
    }
    
    FILE_STATE ||--o{ PENDING_SYNC : "queued when offline"
```

**Example Records:**

**FileState:**
```json
{
  "key": "file:C:\\MainStorage\\photo.jpg",
  "source_path": "C:\\MainStorage\\photo.jpg",
  "hash": "abc123def456...",
  "size": 1048576,
  "last_synced": 1699545600,
  "target_drive": "uuid-1234-5678",
  "target_path": "E:\\images\\photo.jpg",
  "file_category": "images"
}
```

**PendingSync:**
```json
{
  "key": "pending:C:\\MainStorage\\video.mp4",
  "source_path": "C:\\MainStorage\\video.mp4",
  "file_category": "videos",
  "target_drive": "uuid-9876-5432",
  "hash": "xyz789abc...",
  "size": 52428800,
  "created_at": 1699545700
}
```

## 📁 File Category Mapping

```mermaid
mindmap
  root((File Orchestrator))
    Images
      jpg
      jpeg
      png
      gif
      bmp
      webp
      svg
    Videos
      mp4
      avi
      mov
      mkv
      flv
      wmv
      webm
    Music
      mp3
      wav
      flac
      aac
      ogg
      m4a
      wma
    Documents
      pdf
      doc
      docx
      txt
      xlsx
      pptx
    Archives
      zip
      rar
      7z
      tar
      gz
```

**Drive Assignment:**
```mermaid
graph LR
    A[Source HDD] --> B{File Classifier}
    B -->|images| C[ImageUSB]
    B -->|videos| D[VideoUSB]
    B -->|music| E[MusicUSB]
    B -->|documents| F[DocUSB]
    B -->|archives| G[ArchiveUSB]
    
    style A fill:#e3f2fd
    style C fill:#ffcdd2
    style D fill:#f8bbd0
    style E fill:#e1bee7
    style F fill:#d1c4e9
    style G fill:#c5cae9
```

**File Type → USB Mapping:**
```mermaid
flowchart LR
    subgraph SRC["SOURCE (HDD) - MainStorage/"]
        F1[photo1.jpg]
        F2[photo2.png]
        F3[video1.mp4]
        F4[video2.avi]
        F5[song1.mp3]
        F6[song2.flac]
        F7[doc1.pdf]
        F8[archive.zip]
    end
    
    subgraph USB1["USB1 (Images)<br/>E:\images\"]
        I1[ ]
    end
    
    subgraph USB2["USB2 (Videos)<br/>F:\videos\"]
        V1[ ]
    end
    
    subgraph USB3["USB3 (Music)<br/>G:\music\"]
        M1[ ]
    end
    
    F1 --> I1
    F2 --> I1
    F3 --> V1
    F4 --> V1
    F5 --> M1
    F6 --> M1
    
    style SRC fill:#e3f2fd
    style USB1 fill:#ffcdd2
    style USB2 fill:#f8bbd0
    style USB3 fill:#e1bee7
    style I1 fill:#ffcdd2
    style V1 fill:#f8bbd0
    style M1 fill:#e1bee7
```

## ⚙️ Configuration Structure

```mermaid
classDiagram
    class config_toml {
        <<TOML File>>
    }
    
    class source {
        +String path
        Where files come from
    }
    
    class rules {
        +Array~String~ images
        +Array~String~ videos
        +Array~String~ music
        +Array~String~ documents
        +Array~String~ archives
        File extensions per category
    }
    
    class drives {
        +Map~UUID, DriveConfig~
        Drive registration
    }
    
    class DriveConfig {
        +String label
        +String target
        +String path
        UUID → Category mapping
    }
    
    config_toml *-- source
    config_toml *-- rules
    config_toml *-- drives
    drives *-- DriveConfig
```

**Example config.toml:**
```toml
[source]
path = "D:/MainStorage"  # Where files come from

[rules]
images = ["jpg", "png", "gif", "bmp"]  # File extensions
videos = ["mp4", "avi", "mov", "mkv"]  # per category
music  = ["mp3", "wav", "flac", "aac"]

[drives.d158faad-4337-4eeb-a06f-94434eca6d91]
label = "ImageUSB"     # Drive registration
target = "images"      # UUID → Category mapping
path = "E:/"           # Use forward slash
```

## 🔍 Watch Mode Timeline

```mermaid
gantt
    title Watch Mode Operations Timeline
    dateFormat mm:ss
    axisFormat %M:%S
    
    section Startup
    orchestrator run           :00:00, 5s
    Start watching HDD         :00:00, 5s
    Start drive monitor        :00:00, 5s
    
    section File Operations
    User adds photo.jpg        :milestone, 00:05, 0s
    Detect & classify          :00:05, 2s
    Sync to ImageUSB          :00:07, 3s
    
    section Drive Monitoring
    Drive check (10s interval) :00:10, 1s
    All drives connected       :00:10, 1s
    
    section Pending Queue
    User adds video.mp4        :milestone, 00:15, 0s
    VideoUSB disconnected      :00:15, 1s
    Add to pending queue       :00:16, 1s
    Drive check                :00:20, 1s
    VideoUSB still offline     :00:20, 1s
    
    section Queue Processing
    User plugs VideoUSB        :milestone, 00:25, 0s
    Detect USB connection      :00:25, 1s
    Process pending queue      :00:26, 3s
    Sync video.mp4            :00:26, 3s
    
    section Idle
    Drive check                :00:30, 1s
    All synced                 :00:30, 1s
```

## 🎯 Summary

This visual documentation shows:
- ✅ System architecture (layered design)
- ✅ Data flow (from user to disk)
- ✅ Sync process (step-by-step)
- ✅ Database schema (state storage)
- ✅ File mapping (category routing)
- ✅ Configuration (TOML structure)
- ✅ Watch mode timeline (real-time operations)

---

**These diagrams help visualize the complete system! 🎨**
