"""
Snapshot functionality for Eventuali event sourcing.

Provides high-performance aggregate snapshots with compression to improve
reconstruction performance for aggregates with many events.
"""

from typing import Optional, List, Dict, Any
import json
from dataclasses import dataclass

try:
    from . import _PySnapshotService as PySnapshotService
    from . import _PySnapshotConfig as PySnapshotConfig 
    from . import _PyAggregateSnapshot as PyAggregateSnapshot
except ImportError:
    # Fallback for development/testing when Rust bindings aren't available
    PySnapshotService = None
    PySnapshotConfig = None 
    PyAggregateSnapshot = None


@dataclass
class SnapshotConfig:
    """Configuration for aggregate snapshot behavior."""
    
    snapshot_frequency: int = 100  # Take snapshot every N events
    max_snapshot_age_hours: int = 168  # 7 days
    compression: str = "gzip"  # none, gzip, lz4
    auto_cleanup: bool = True
    
    def to_rust(self) -> "PySnapshotConfig":
        """Convert to Rust snapshot config."""
        if PySnapshotConfig is None:
            raise RuntimeError("Rust bindings not available")
        return PySnapshotConfig(
            self.snapshot_frequency,
            self.max_snapshot_age_hours, 
            self.compression,
            self.auto_cleanup
        )


class AggregateSnapshot:
    """Represents a snapshot of an aggregate at a specific version."""
    
    def __init__(self, rust_snapshot: "PyAggregateSnapshot"):
        self._rust_snapshot = rust_snapshot
    
    @property
    def snapshot_id(self) -> str:
        """Unique identifier for this snapshot."""
        return self._rust_snapshot.snapshot_id
    
    @property
    def aggregate_id(self) -> str:
        """ID of the aggregate this snapshot represents."""
        return self._rust_snapshot.aggregate_id
    
    @property
    def aggregate_type(self) -> str:
        """Type of the aggregate."""
        return self._rust_snapshot.aggregate_type
    
    @property
    def aggregate_version(self) -> int:
        """Version of the aggregate when snapshot was taken."""
        return self._rust_snapshot.aggregate_version
    
    @property
    def state_data(self) -> bytes:
        """Raw snapshot state data."""
        return bytes(self._rust_snapshot.state_data)
    
    @property
    def compression(self) -> str:
        """Compression algorithm used."""
        return self._rust_snapshot.compression
    
    @property
    def created_at(self) -> str:
        """When this snapshot was created (ISO format)."""
        return self._rust_snapshot.created_at
    
    @property
    def original_size(self) -> int:
        """Size of original data before compression."""
        return self._rust_snapshot.original_size
    
    @property
    def compressed_size(self) -> int:
        """Size of compressed data."""
        return self._rust_snapshot.compressed_size
    
    @property
    def event_count(self) -> int:
        """Number of events used to build this snapshot."""
        return self._rust_snapshot.event_count
    
    @property
    def checksum(self) -> str:
        """Data integrity checksum."""
        return self._rust_snapshot.checksum
    
    @property
    def compression_ratio(self) -> float:
        """Compression ratio (compressed/original)."""
        if self.original_size == 0:
            return 1.0
        return self.compressed_size / self.original_size
    
    def __repr__(self) -> str:
        return (f"AggregateSnapshot(id={self.snapshot_id[:8]}..., "
                f"aggregate_id={self.aggregate_id}, version={self.aggregate_version}, "
                f"size={self.compressed_size}, compression={self.compression})")


class SnapshotService:
    """Service for managing aggregate snapshots with high performance."""
    
    def __init__(self, config: Optional[SnapshotConfig] = None):
        """Initialize snapshot service.
        
        Args:
            config: Snapshot configuration. Uses defaults if None.
        """
        if PySnapshotService is None:
            raise RuntimeError("Rust bindings not available. Please build with 'uv run maturin develop --release'")
        
        self.config = config or SnapshotConfig()
        self._rust_service = PySnapshotService()
        self._initialized = False
    
    def initialize(self, database_url: str) -> None:
        """Initialize the snapshot service with database connection.
        
        Args:
            database_url: SQLite database URL (e.g., 'sqlite:///snapshots.db' or 'sqlite://:memory:')
        """
        rust_config = self.config.to_rust()
        self._rust_service.initialize(database_url, rust_config)
        self._initialized = True
    
    def create_snapshot(
        self,
        aggregate_id: str,
        aggregate_type: str, 
        aggregate_version: int,
        state_data: bytes,
        event_count: int
    ) -> AggregateSnapshot:
        """Create a snapshot from aggregate state data.
        
        Args:
            aggregate_id: ID of the aggregate
            aggregate_type: Type of the aggregate
            aggregate_version: Current version of the aggregate
            state_data: Serialized aggregate state
            event_count: Number of events used to build this state
            
        Returns:
            Created aggregate snapshot
        """
        self._ensure_initialized()
        
        rust_snapshot = self._rust_service.create_snapshot(
            aggregate_id,
            aggregate_type,
            aggregate_version,
            state_data,
            event_count
        )
        
        return AggregateSnapshot(rust_snapshot)
    
    def load_latest_snapshot(self, aggregate_id: str) -> Optional[AggregateSnapshot]:
        """Load the most recent snapshot for an aggregate.
        
        Args:
            aggregate_id: ID of the aggregate
            
        Returns:
            Latest snapshot or None if no snapshots exist
        """
        self._ensure_initialized()
        
        rust_snapshot = self._rust_service.load_latest_snapshot(aggregate_id)
        if rust_snapshot is None:
            return None
        
        return AggregateSnapshot(rust_snapshot)
    
    def decompress_snapshot_data(self, snapshot: AggregateSnapshot) -> bytes:
        """Decompress snapshot state data.
        
        Args:
            snapshot: Aggregate snapshot
            
        Returns:
            Decompressed state data
        """
        self._ensure_initialized()
        return bytes(self._rust_service.decompress_snapshot_data(snapshot._rust_snapshot))
    
    def should_take_snapshot(self, aggregate_id: str, current_version: int) -> bool:
        """Check if a snapshot should be taken for the current aggregate state.
        
        Args:
            aggregate_id: ID of the aggregate
            current_version: Current version of the aggregate
            
        Returns:
            True if a snapshot should be taken
        """
        self._ensure_initialized()
        return self._rust_service.should_take_snapshot(aggregate_id, current_version)
    
    def cleanup_old_snapshots(self) -> int:
        """Clean up old snapshots based on configuration.
        
        Returns:
            Number of snapshots cleaned up
        """
        self._ensure_initialized()
        return self._rust_service.cleanup_old_snapshots()
    
    def _ensure_initialized(self) -> None:
        """Ensure the service is initialized."""
        if not self._initialized:
            raise RuntimeError("SnapshotService must be initialized with a database URL first")
    
    def __repr__(self) -> str:
        status = "initialized" if self._initialized else "not initialized"
        return f"SnapshotService({status}, config={self.config})"


# Helper functions for common snapshot patterns

def create_json_snapshot(
    service: SnapshotService,
    aggregate_id: str,
    aggregate_type: str,
    aggregate_version: int,
    state_dict: Dict[str, Any],
    event_count: int
) -> AggregateSnapshot:
    """Create a snapshot from a JSON-serializable state dictionary.
    
    Args:
        service: Snapshot service instance
        aggregate_id: ID of the aggregate
        aggregate_type: Type of the aggregate
        aggregate_version: Current version of the aggregate
        state_dict: State as a JSON-serializable dictionary
        event_count: Number of events used to build this state
        
    Returns:
        Created aggregate snapshot
    """
    state_json = json.dumps(state_dict, sort_keys=True)
    state_data = state_json.encode('utf-8')
    
    return service.create_snapshot(
        aggregate_id,
        aggregate_type,
        aggregate_version,
        state_data,
        event_count
    )


def load_json_snapshot_state(
    service: SnapshotService,
    snapshot: AggregateSnapshot
) -> Dict[str, Any]:
    """Load and deserialize JSON state from a snapshot.
    
    Args:
        service: Snapshot service instance
        snapshot: Aggregate snapshot
        
    Returns:
        Deserialized state dictionary
    """
    decompressed_data = service.decompress_snapshot_data(snapshot)
    state_json = decompressed_data.decode('utf-8')
    return json.loads(state_json)


__all__ = [
    "SnapshotService",
    "SnapshotConfig", 
    "AggregateSnapshot",
    "create_json_snapshot",
    "load_json_snapshot_state",
]