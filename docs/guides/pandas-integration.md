# Pandas Integration Guide

**Data analysis and reporting with Eventuali event streams**

This guide demonstrates how to integrate Eventuali with Pandas for building powerful data analysis workflows, real-time analytics, and reporting systems from event streams.

## 🎯 What You'll Learn

- ✅ Converting event streams to Pandas DataFrames
- ✅ Real-time analytics with streaming data
- ✅ Time-series analysis of domain events
- ✅ Business intelligence dashboards
- ✅ Event-driven ETL pipelines
- ✅ Performance optimization for large datasets
- ✅ Integration with Jupyter notebooks

## 📋 Prerequisites

```bash
# Install dependencies
uv add pandas numpy eventuali
uv add matplotlib seaborn plotly  # For visualization
uv add jupyter ipywidgets  # For notebooks
uv add sqlalchemy psycopg2-binary  # For database integration
uv add pyarrow fastparquet  # For Parquet support
```

## 🚀 Basic Integration

### Event Stream to DataFrame Conversion

Create `analytics/stream_converter.py`:

```python
import pandas as pd
import numpy as np
from typing import List, Dict, Any, Optional, AsyncGenerator
from datetime import datetime, timedelta
import asyncio

from eventuali import EventStore, Event
from eventuali.streaming import EventStreamer, Subscription, EventStreamReceiver
from eventuali.exceptions import EventualiError

class EventStreamAnalyzer:
    """Convert event streams to Pandas DataFrames for analysis."""
    
    def __init__(self, event_store: EventStore, event_streamer: Optional[EventStreamer] = None):
        self.event_store = event_store
        self.event_streamer = event_streamer or EventStreamer(capacity=10000)
        self._data_cache = {}
    
    async def events_to_dataframe(
        self, 
        aggregate_ids: Optional[List[str]] = None,
        event_types: Optional[List[str]] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        aggregate_type: Optional[str] = None
    ) -> pd.DataFrame:
        """Convert events to Pandas DataFrame with filtering options."""
        
        events_data = []
        
        if aggregate_ids:
            # Load events for specific aggregates
            for aggregate_id in aggregate_ids:
                events = await self.event_store.load_events(aggregate_id)
                events_data.extend(self._events_to_records(events))
        else:
            # For all events, we'd need a different approach
            # This is a simplified version
            raise NotImplementedError("Loading all events requires custom implementation")
        
        # Create DataFrame
        df = pd.DataFrame(events_data)
        
        if df.empty:
            return df
        
        # Convert timestamp to datetime
        df['timestamp'] = pd.to_datetime(df['timestamp'])
        
        # Apply filters
        if event_types:
            df = df[df['event_type'].isin(event_types)]
        
        if aggregate_type:
            df = df[df['aggregate_type'] == aggregate_type]
        
        if start_date:
            df = df[df['timestamp'] >= start_date]
        
        if end_date:
            df = df[df['timestamp'] <= end_date]
        
        # Sort by timestamp
        df = df.sort_values('timestamp').reset_index(drop=True)
        
        return df
    
    def _events_to_records(self, events: List[Event]) -> List[Dict[str, Any]]:
        """Convert events to records suitable for DataFrame."""
        records = []
        
        for event in events:
            record = {
                'aggregate_id': event.aggregate_id,
                'aggregate_type': event.aggregate_type,
                'aggregate_version': event.aggregate_version,
                'event_type': event.event_type,
                'timestamp': event.timestamp,
                'global_position': getattr(event, 'global_position', None),
            }
            
            # Flatten event data
            event_data = event.to_dict()
            for key, value in event_data.items():
                if key not in record:
                    # Prefix custom fields to avoid conflicts
                    record[f'data_{key}'] = value
            
            records.append(record)
        
        return records
    
    async def stream_to_dataframe(
        self, 
        subscription: Subscription,
        max_events: int = 1000,
        timeout_seconds: int = 30
    ) -> pd.DataFrame:
        """Stream events to DataFrame with limits."""
        
        receiver = await self.event_streamer.subscribe(subscription)
        events_data = []
        
        try:
            count = 0
            start_time = asyncio.get_event_loop().time()
            
            async for stream_event in receiver:
                event = stream_event.event
                
                # Convert to record
                record = self._events_to_records([event])[0]
                record['stream_position'] = stream_event.stream_position
                record['global_position'] = stream_event.global_position
                
                events_data.append(record)
                count += 1
                
                # Check limits
                if count >= max_events:
                    break
                
                if asyncio.get_event_loop().time() - start_time > timeout_seconds:
                    break
            
        except Exception as e:
            print(f"Error streaming events: {e}")
        finally:
            await self.event_streamer.unsubscribe(subscription.id)
        
        # Create DataFrame
        df = pd.DataFrame(events_data)
        if not df.empty:
            df['timestamp'] = pd.to_datetime(df['timestamp'])
            df = df.sort_values('timestamp').reset_index(drop=True)
        
        return df

class RealTimeAnalytics:
    """Real-time analytics using streaming events."""
    
    def __init__(self, event_streamer: EventStreamer):
        self.event_streamer = event_streamer
        self.metrics = {}
        self.data_buffer = []
        self.buffer_size = 1000
    
    async def start_analytics(self, subscription: Subscription):
        """Start real-time analytics processing."""
        receiver = await self.event_streamer.subscribe(subscription)
        
        async for stream_event in receiver:
            await self._process_event(stream_event.event)
    
    async def _process_event(self, event: Event):
        """Process individual event for analytics."""
        # Add to buffer
        self.data_buffer.append({
            'timestamp': event.timestamp,
            'event_type': event.event_type,
            'aggregate_type': event.aggregate_type,
            'data': event.to_dict()
        })
        
        # Trim buffer if too large
        if len(self.data_buffer) > self.buffer_size:
            self.data_buffer = self.data_buffer[-self.buffer_size:]
        
        # Update metrics
        await self._update_metrics(event)
    
    async def _update_metrics(self, event: Event):
        """Update real-time metrics."""
        now = datetime.now()
        
        # Event count by type
        if 'event_counts' not in self.metrics:
            self.metrics['event_counts'] = {}
        
        event_type = event.event_type
        if event_type not in self.metrics['event_counts']:
            self.metrics['event_counts'][event_type] = 0
        self.metrics['event_counts'][event_type] += 1
        
        # Events per minute
        minute_key = now.strftime('%Y-%m-%d %H:%M')
        if 'events_per_minute' not in self.metrics:
            self.metrics['events_per_minute'] = {}
        if minute_key not in self.metrics['events_per_minute']:
            self.metrics['events_per_minute'][minute_key] = 0
        self.metrics['events_per_minute'][minute_key] += 1
    
    def get_current_dataframe(self) -> pd.DataFrame:
        """Get current buffer as DataFrame."""
        if not self.data_buffer:
            return pd.DataFrame()
        
        df = pd.DataFrame(self.data_buffer)
        df['timestamp'] = pd.to_datetime(df['timestamp'])
        return df.sort_values('timestamp').reset_index(drop=True)
    
    def get_metrics_dataframe(self) -> Dict[str, pd.DataFrame]:
        """Get current metrics as DataFrames."""
        metrics_dfs = {}
        
        # Event counts
        if 'event_counts' in self.metrics:
            metrics_dfs['event_counts'] = pd.DataFrame([
                {'event_type': k, 'count': v} 
                for k, v in self.metrics['event_counts'].items()
            ])
        
        # Events per minute
        if 'events_per_minute' in self.metrics:
            metrics_dfs['events_per_minute'] = pd.DataFrame([
                {'minute': k, 'events': v}
                for k, v in self.metrics['events_per_minute'].items()
            ])
            if not metrics_dfs['events_per_minute'].empty:
                metrics_dfs['events_per_minute']['minute'] = pd.to_datetime(
                    metrics_dfs['events_per_minute']['minute']
                )
        
        return metrics_dfs
```

## 📊 Business Analytics Examples

### User Analytics

Create `analytics/user_analytics.py`:

```python
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from datetime import datetime, timedelta
from typing import Dict, List, Tuple

class UserAnalytics:
    """Analytics for user behavior and lifecycle."""
    
    def __init__(self, events_df: pd.DataFrame):
        self.events_df = events_df.copy()
        self._prepare_data()
    
    def _prepare_data(self):
        """Prepare data for analysis."""
        # Ensure timestamp is datetime
        self.events_df['timestamp'] = pd.to_datetime(self.events_df['timestamp'])
        
        # Extract date components
        self.events_df['date'] = self.events_df['timestamp'].dt.date
        self.events_df['hour'] = self.events_df['timestamp'].dt.hour
        self.events_df['day_of_week'] = self.events_df['timestamp'].dt.day_name()
        self.events_df['month'] = self.events_df['timestamp'].dt.month
        
        # Sort by timestamp
        self.events_df = self.events_df.sort_values('timestamp').reset_index(drop=True)
    
    def user_registration_analysis(self) -> Dict[str, pd.DataFrame]:
        """Analyze user registration patterns."""
        registration_events = self.events_df[
            self.events_df['event_type'] == 'UserRegistered'
        ].copy()
        
        if registration_events.empty:
            return {}
        
        results = {}
        
        # Daily registrations
        daily_registrations = registration_events.groupby('date').size().reset_index(name='registrations')
        daily_registrations['date'] = pd.to_datetime(daily_registrations['date'])
        results['daily_registrations'] = daily_registrations
        
        # Hourly patterns
        hourly_registrations = registration_events.groupby('hour').size().reset_index(name='registrations')
        results['hourly_patterns'] = hourly_registrations
        
        # Day of week patterns
        dow_registrations = registration_events.groupby('day_of_week').size().reset_index(name='registrations')
        # Order by day of week
        dow_order = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']
        dow_registrations['day_of_week'] = pd.Categorical(
            dow_registrations['day_of_week'], 
            categories=dow_order, 
            ordered=True
        )
        dow_registrations = dow_registrations.sort_values('day_of_week').reset_index(drop=True)
        results['day_of_week_patterns'] = dow_registrations
        
        # Monthly trends
        monthly_registrations = registration_events.groupby('month').size().reset_index(name='registrations')
        results['monthly_trends'] = monthly_registrations
        
        return results
    
    def user_activity_analysis(self) -> Dict[str, pd.DataFrame]:
        """Analyze user activity patterns."""
        results = {}
        
        # Activity by user
        user_activity = self.events_df.groupby('aggregate_id').agg({
            'event_type': 'count',
            'timestamp': ['min', 'max']
        }).reset_index()
        
        user_activity.columns = ['user_id', 'total_events', 'first_event', 'last_event']
        user_activity['days_active'] = (
            user_activity['last_event'] - user_activity['first_event']
        ).dt.days + 1
        user_activity['events_per_day'] = user_activity['total_events'] / user_activity['days_active']
        
        results['user_activity_summary'] = user_activity
        
        # Most active users
        most_active = user_activity.nlargest(10, 'total_events')
        results['most_active_users'] = most_active
        
        # Activity distribution
        activity_stats = user_activity['total_events'].describe()
        results['activity_distribution'] = pd.DataFrame([activity_stats])
        
        # Event type distribution
        event_type_dist = self.events_df['event_type'].value_counts().reset_index()
        event_type_dist.columns = ['event_type', 'count']
        results['event_type_distribution'] = event_type_dist
        
        return results
    
    def user_journey_analysis(self) -> Dict[str, pd.DataFrame]:
        """Analyze user journey and conversion funnels."""
        results = {}
        
        # User lifecycle stages
        lifecycle_events = ['UserRegistered', 'UserEmailChanged', 'UserProfileUpdated', 'UserDeactivated']
        
        user_journeys = []
        for user_id in self.events_df['aggregate_id'].unique():
            user_events = self.events_df[
                self.events_df['aggregate_id'] == user_id
            ].sort_values('timestamp')
            
            journey = {
                'user_id': user_id,
                'total_events': len(user_events),
                'first_event': user_events.iloc[0]['timestamp'],
                'last_event': user_events.iloc[-1]['timestamp'],
                'lifecycle_stages': len(user_events['event_type'].unique()),
            }
            
            # Track specific lifecycle events
            for event_type in lifecycle_events:
                journey[f'has_{event_type.lower()}'] = event_type in user_events['event_type'].values
                event_occurrences = user_events[user_events['event_type'] == event_type]
                if not event_occurrences.empty:
                    journey[f'{event_type.lower()}_timestamp'] = event_occurrences.iloc[0]['timestamp']
            
            user_journeys.append(journey)
        
        journey_df = pd.DataFrame(user_journeys)
        results['user_journeys'] = journey_df
        
        # Conversion funnel
        funnel_data = []
        for event_type in lifecycle_events:
            count = journey_df[f'has_{event_type.lower()}'].sum()
            percentage = (count / len(journey_df)) * 100
            funnel_data.append({
                'stage': event_type,
                'users': count,
                'percentage': percentage
            })
        
        results['conversion_funnel'] = pd.DataFrame(funnel_data)
        
        return results
    
    def churn_analysis(self) -> Dict[str, pd.DataFrame]:
        """Analyze user churn patterns."""
        results = {}
        
        # Define churn as no activity in last 30 days
        cutoff_date = datetime.now() - timedelta(days=30)
        
        user_last_activity = self.events_df.groupby('aggregate_id')['timestamp'].max().reset_index()
        user_last_activity.columns = ['user_id', 'last_activity']
        
        user_last_activity['is_churned'] = user_last_activity['last_activity'] < cutoff_date
        user_last_activity['days_since_activity'] = (
            datetime.now() - user_last_activity['last_activity']
        ).dt.days
        
        results['churn_status'] = user_last_activity
        
        # Churn rate
        total_users = len(user_last_activity)
        churned_users = user_last_activity['is_churned'].sum()
        churn_rate = (churned_users / total_users) * 100
        
        results['churn_metrics'] = pd.DataFrame([{
            'total_users': total_users,
            'churned_users': churned_users,
            'active_users': total_users - churned_users,
            'churn_rate_percentage': churn_rate
        }])
        
        return results
    
    def cohort_analysis(self) -> pd.DataFrame:
        """Perform cohort analysis based on registration month."""
        # Get user registration dates
        registration_events = self.events_df[
            self.events_df['event_type'] == 'UserRegistered'
        ][['aggregate_id', 'timestamp']].copy()
        
        if registration_events.empty:
            return pd.DataFrame()
        
        registration_events['registration_month'] = registration_events['timestamp'].dt.to_period('M')
        
        # Get all user activity
        user_activity = self.events_df.groupby('aggregate_id')['timestamp'].apply(
            lambda x: x.dt.to_period('M').unique()
        ).reset_index()
        
        # Build cohort table
        cohort_data = []
        
        for _, user_reg in registration_events.iterrows():
            user_id = user_reg['aggregate_id']
            reg_month = user_reg['registration_month']
            
            user_months = user_activity[user_activity['aggregate_id'] == user_id]['timestamp'].iloc[0]
            
            for month in user_months:
                months_since_reg = (month - reg_month).n
                cohort_data.append({
                    'user_id': user_id,
                    'registration_month': reg_month,
                    'activity_month': month,
                    'months_since_registration': months_since_reg
                })
        
        cohort_df = pd.DataFrame(cohort_data)
        
        # Create cohort table
        cohort_table = cohort_df.groupby(['registration_month', 'months_since_registration']).size().reset_index(name='users')
        cohort_pivot = cohort_table.pivot(
            index='registration_month', 
            columns='months_since_registration', 
            values='users'
        ).fillna(0)
        
        # Calculate retention rates
        cohort_sizes = registration_events.groupby('registration_month').size()
        retention_table = cohort_pivot.divide(cohort_sizes, axis=0)
        
        return retention_table

def visualize_user_analytics(analytics: UserAnalytics, save_path: str = None):
    """Create comprehensive visualizations for user analytics."""
    
    # Set style
    plt.style.use('seaborn-v0_8')
    fig, axes = plt.subplots(2, 3, figsize=(20, 12))
    fig.suptitle('User Analytics Dashboard', fontsize=16, fontweight='bold')
    
    # 1. Daily registrations
    reg_analysis = analytics.user_registration_analysis()
    if 'daily_registrations' in reg_analysis:
        daily_reg = reg_analysis['daily_registrations']
        axes[0, 0].plot(daily_reg['date'], daily_reg['registrations'], marker='o')
        axes[0, 0].set_title('Daily User Registrations')
        axes[0, 0].set_xlabel('Date')
        axes[0, 0].set_ylabel('Registrations')
        axes[0, 0].tick_params(axis='x', rotation=45)
    
    # 2. Hourly registration patterns
    if 'hourly_patterns' in reg_analysis:
        hourly_reg = reg_analysis['hourly_patterns']
        axes[0, 1].bar(hourly_reg['hour'], hourly_reg['registrations'])
        axes[0, 1].set_title('Registration Patterns by Hour')
        axes[0, 1].set_xlabel('Hour of Day')
        axes[0, 1].set_ylabel('Registrations')
    
    # 3. Event type distribution
    activity_analysis = analytics.user_activity_analysis()
    if 'event_type_distribution' in activity_analysis:
        event_dist = activity_analysis['event_type_distribution']
        axes[0, 2].pie(event_dist['count'], labels=event_dist['event_type'], autopct='%1.1f%%')
        axes[0, 2].set_title('Event Type Distribution')
    
    # 4. User activity distribution
    if 'user_activity_summary' in activity_analysis:
        activity_summary = activity_analysis['user_activity_summary']
        axes[1, 0].hist(activity_summary['total_events'], bins=20, alpha=0.7)
        axes[1, 0].set_title('User Activity Distribution')
        axes[1, 0].set_xlabel('Total Events per User')
        axes[1, 0].set_ylabel('Number of Users')
    
    # 5. Conversion funnel
    journey_analysis = analytics.user_journey_analysis()
    if 'conversion_funnel' in journey_analysis:
        funnel = journey_analysis['conversion_funnel']
        axes[1, 1].bar(range(len(funnel)), funnel['users'])
        axes[1, 1].set_title('User Conversion Funnel')
        axes[1, 1].set_xlabel('Lifecycle Stage')
        axes[1, 1].set_ylabel('Number of Users')
        axes[1, 1].set_xticks(range(len(funnel)))
        axes[1, 1].set_xticklabels(funnel['stage'], rotation=45)
    
    # 6. Cohort heatmap
    cohort_data = analytics.cohort_analysis()
    if not cohort_data.empty:
        sns.heatmap(cohort_data, annot=True, fmt='.2f', cmap='YlOrRd', ax=axes[1, 2])
        axes[1, 2].set_title('Cohort Retention Analysis')
        axes[1, 2].set_xlabel('Months Since Registration')
        axes[1, 2].set_ylabel('Registration Month')
    
    plt.tight_layout()
    
    if save_path:
        plt.savefig(save_path, dpi=300, bbox_inches='tight')
    
    plt.show()
```

## 📈 Real-Time Dashboard

Create `analytics/dashboard.py`:

```python
import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
import asyncio
from datetime import datetime, timedelta
from typing import Dict, Any
import json

from .stream_converter import RealTimeAnalytics, EventStreamAnalyzer

class RealTimeDashboard:
    """Real-time analytics dashboard with Plotly."""
    
    def __init__(self, analytics: RealTimeAnalytics):
        self.analytics = analytics
        self.last_update = datetime.now()
        self.update_interval = 5  # seconds
    
    def create_dashboard(self) -> Dict[str, Any]:
        """Create comprehensive dashboard with multiple charts."""
        
        # Get current data
        current_df = self.analytics.get_current_dataframe()
        metrics_dfs = self.analytics.get_metrics_dataframe()
        
        dashboard = {}
        
        # 1. Event timeline
        if not current_df.empty:
            dashboard['event_timeline'] = self._create_event_timeline(current_df)
        
        # 2. Event type distribution
        if not current_df.empty:
            dashboard['event_distribution'] = self._create_event_distribution(current_df)
        
        # 3. Events per minute
        if 'events_per_minute' in metrics_dfs and not metrics_dfs['events_per_minute'].empty:
            dashboard['events_per_minute'] = self._create_events_per_minute(
                metrics_dfs['events_per_minute']
            )
        
        # 4. Real-time metrics
        dashboard['current_metrics'] = self._create_metrics_summary()
        
        return dashboard
    
    def _create_event_timeline(self, df: pd.DataFrame) -> go.Figure:
        """Create event timeline chart."""
        # Group events by minute
        df['minute'] = df['timestamp'].dt.floor('T')
        timeline_data = df.groupby(['minute', 'event_type']).size().reset_index(name='count')
        
        fig = px.line(
            timeline_data, 
            x='minute', 
            y='count', 
            color='event_type',
            title='Event Timeline (Real-time)',
            labels={'minute': 'Time', 'count': 'Event Count'}
        )
        
        fig.update_layout(
            xaxis_title='Time',
            yaxis_title='Event Count',
            hovermode='x unified',
            showlegend=True
        )
        
        return fig
    
    def _create_event_distribution(self, df: pd.DataFrame) -> go.Figure:
        """Create event type distribution chart."""
        event_counts = df['event_type'].value_counts()
        
        fig = go.Figure(data=[
            go.Pie(
                labels=event_counts.index,
                values=event_counts.values,
                title="Current Event Distribution"
            )
        ])
        
        fig.update_layout(
            title_text="Event Type Distribution",
            showlegend=True
        )
        
        return fig
    
    def _create_events_per_minute(self, epm_df: pd.DataFrame) -> go.Figure:
        """Create events per minute chart."""
        fig = go.Figure()
        
        fig.add_trace(go.Scatter(
            x=epm_df['minute'],
            y=epm_df['events'],
            mode='lines+markers',
            name='Events/Minute',
            line=dict(color='blue', width=2),
            marker=dict(size=6)
        ))
        
        fig.update_layout(
            title='Events per Minute',
            xaxis_title='Time',
            yaxis_title='Events',
            hovermode='x unified',
            showlegend=True
        )
        
        return fig
    
    def _create_metrics_summary(self) -> Dict[str, Any]:
        """Create current metrics summary."""
        metrics = self.analytics.metrics
        
        summary = {
            'total_events': sum(metrics.get('event_counts', {}).values()),
            'event_types': len(metrics.get('event_counts', {})),
            'current_rate': self._calculate_current_rate(),
            'last_update': self.last_update.isoformat(),
            'buffer_size': len(self.analytics.data_buffer)
        }
        
        return summary
    
    def _calculate_current_rate(self) -> float:
        """Calculate current event rate (events per second)."""
        epm_data = self.analytics.metrics.get('events_per_minute', {})
        if not epm_data:
            return 0.0
        
        # Get last minute's count
        recent_minutes = sorted(epm_data.keys())[-5:]  # Last 5 minutes
        recent_counts = [epm_data[minute] for minute in recent_minutes]
        
        if not recent_counts:
            return 0.0
        
        avg_per_minute = sum(recent_counts) / len(recent_counts)
        return round(avg_per_minute / 60, 2)  # Convert to per second
    
    async def start_live_dashboard(self, update_callback=None):
        """Start live dashboard updates."""
        while True:
            try:
                dashboard_data = self.create_dashboard()
                self.last_update = datetime.now()
                
                if update_callback:
                    await update_callback(dashboard_data)
                
                await asyncio.sleep(self.update_interval)
                
            except Exception as e:
                print(f"Dashboard update error: {e}")
                await asyncio.sleep(self.update_interval)

class JupyterDashboard:
    """Jupyter notebook integration for interactive analytics."""
    
    def __init__(self, analyzer: EventStreamAnalyzer):
        self.analyzer = analyzer
    
    def create_interactive_widgets(self):
        """Create interactive widgets for Jupyter notebooks."""
        try:
            import ipywidgets as widgets
            from IPython.display import display
        except ImportError:
            print("ipywidgets not installed. Run: uv add ipywidgets")
            return
        
        # Date range picker
        date_picker = widgets.DatePicker(
            description='Start Date:',
            value=datetime.now().date() - timedelta(days=30)
        )
        
        end_date_picker = widgets.DatePicker(
            description='End Date:',
            value=datetime.now().date()
        )
        
        # Event type filter
        event_type_filter = widgets.SelectMultiple(
            options=['UserRegistered', 'UserEmailChanged', 'UserProfileUpdated', 'UserDeactivated'],
            description='Event Types:',
            disabled=False
        )
        
        # Aggregate type filter
        aggregate_type_filter = widgets.Dropdown(
            options=['User', 'Order', 'Product', 'All'],
            value='User',
            description='Aggregate Type:'
        )
        
        # Analysis type
        analysis_type = widgets.Dropdown(
            options=[
                'Registration Analysis', 
                'Activity Analysis', 
                'Journey Analysis',
                'Churn Analysis',
                'Cohort Analysis'
            ],
            value='Registration Analysis',
            description='Analysis Type:'
        )
        
        # Run button
        run_button = widgets.Button(
            description='Run Analysis',
            button_style='primary',
            tooltip='Click to run analysis'
        )
        
        # Output area
        output = widgets.Output()
        
        def on_run_clicked(b):
            """Handle run button click."""
            with output:
                output.clear_output()
                try:
                    # This would need to be implemented as async in a real scenario
                    print("Running analysis...")
                    print(f"Date range: {date_picker.value} to {end_date_picker.value}")
                    print(f"Event types: {list(event_type_filter.value)}")
                    print(f"Aggregate type: {aggregate_type_filter.value}")
                    print(f"Analysis type: {analysis_type.value}")
                    
                    # In a real implementation, you'd run the actual analysis here
                    # and display the results
                    
                except Exception as e:
                    print(f"Error running analysis: {e}")
        
        run_button.on_click(on_run_clicked)
        
        # Layout
        controls = widgets.VBox([
            widgets.HBox([date_picker, end_date_picker]),
            event_type_filter,
            aggregate_type_filter,
            analysis_type,
            run_button
        ])
        
        display(widgets.VBox([controls, output]))
        
        return {
            'date_picker': date_picker,
            'end_date_picker': end_date_picker,
            'event_type_filter': event_type_filter,
            'aggregate_type_filter': aggregate_type_filter,
            'analysis_type': analysis_type,
            'run_button': run_button,
            'output': output
        }
```

## 💾 Data Export and ETL

Create `analytics/data_export.py`:

```python
import pandas as pd
import numpy as np
from typing import Dict, List, Optional, Union
from datetime import datetime
import asyncio
from pathlib import Path

from eventuali import EventStore
from .stream_converter import EventStreamAnalyzer

class DataExporter:
    """Export event data to various formats for external analysis."""
    
    def __init__(self, analyzer: EventStreamAnalyzer):
        self.analyzer = analyzer
    
    async def export_to_csv(
        self, 
        output_path: str,
        aggregate_ids: Optional[List[str]] = None,
        event_types: Optional[List[str]] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None
    ) -> str:
        """Export events to CSV format."""
        
        df = await self.analyzer.events_to_dataframe(
            aggregate_ids=aggregate_ids,
            event_types=event_types,
            start_date=start_date,
            end_date=end_date
        )
        
        df.to_csv(output_path, index=False)
        return output_path
    
    async def export_to_parquet(
        self,
        output_path: str,
        aggregate_ids: Optional[List[str]] = None,
        event_types: Optional[List[str]] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        compression: str = 'snappy'
    ) -> str:
        """Export events to Parquet format for efficient storage."""
        
        df = await self.analyzer.events_to_dataframe(
            aggregate_ids=aggregate_ids,
            event_types=event_types,
            start_date=start_date,
            end_date=end_date
        )
        
        df.to_parquet(output_path, compression=compression, index=False)
        return output_path
    
    async def export_to_excel(
        self,
        output_path: str,
        include_analytics: bool = True,
        aggregate_ids: Optional[List[str]] = None,
        event_types: Optional[List[str]] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None
    ) -> str:
        """Export events and analytics to Excel with multiple sheets."""
        
        df = await self.analyzer.events_to_dataframe(
            aggregate_ids=aggregate_ids,
            event_types=event_types,
            start_date=start_date,
            end_date=end_date
        )
        
        with pd.ExcelWriter(output_path, engine='openpyxl') as writer:
            # Raw events
            df.to_excel(writer, sheet_name='Events', index=False)
            
            if include_analytics and not df.empty:
                from .user_analytics import UserAnalytics
                
                analytics = UserAnalytics(df)
                
                # Registration analysis
                reg_analysis = analytics.user_registration_analysis()
                for name, data in reg_analysis.items():
                    sheet_name = f'Reg_{name[:25]}'  # Excel sheet name limit
                    data.to_excel(writer, sheet_name=sheet_name, index=False)
                
                # Activity analysis
                activity_analysis = analytics.user_activity_analysis()
                for name, data in activity_analysis.items():
                    sheet_name = f'Act_{name[:25]}'
                    data.to_excel(writer, sheet_name=sheet_name, index=False)
                
                # Journey analysis
                journey_analysis = analytics.user_journey_analysis()
                for name, data in journey_analysis.items():
                    sheet_name = f'Journey_{name[:20]}'
                    data.to_excel(writer, sheet_name=sheet_name, index=False)
        
        return output_path
    
    async def export_to_sql_database(
        self,
        connection_string: str,
        table_name: str = 'events_export',
        aggregate_ids: Optional[List[str]] = None,
        event_types: Optional[List[str]] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        if_exists: str = 'replace'
    ) -> str:
        """Export events to SQL database."""
        
        try:
            from sqlalchemy import create_engine
        except ImportError:
            raise ImportError("sqlalchemy required for SQL export. Run: uv add sqlalchemy")
        
        df = await self.analyzer.events_to_dataframe(
            aggregate_ids=aggregate_ids,
            event_types=event_types,
            start_date=start_date,
            end_date=end_date
        )
        
        engine = create_engine(connection_string)
        df.to_sql(table_name, engine, if_exists=if_exists, index=False)
        
        return f"Exported {len(df)} rows to {table_name}"
    
    async def create_data_warehouse_tables(
        self,
        connection_string: str,
        schema_name: str = 'eventuali_analytics'
    ) -> Dict[str, str]:
        """Create optimized data warehouse tables."""
        
        try:
            from sqlalchemy import create_engine, text
        except ImportError:
            raise ImportError("sqlalchemy required. Run: uv add sqlalchemy")
        
        engine = create_engine(connection_string)
        
        # Create schema
        with engine.connect() as conn:
            conn.execute(text(f"CREATE SCHEMA IF NOT EXISTS {schema_name}"))
            conn.commit()
        
        # Define table schemas
        tables = {
            'events_fact': f"""
            CREATE TABLE IF NOT EXISTS {schema_name}.events_fact (
                event_id SERIAL PRIMARY KEY,
                aggregate_id VARCHAR(255) NOT NULL,
                aggregate_type VARCHAR(100) NOT NULL,
                aggregate_version INTEGER NOT NULL,
                event_type VARCHAR(100) NOT NULL,
                timestamp TIMESTAMP NOT NULL,
                global_position BIGINT,
                event_data JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                INDEX idx_aggregate_id (aggregate_id),
                INDEX idx_event_type (event_type),
                INDEX idx_timestamp (timestamp),
                INDEX idx_aggregate_type (aggregate_type)
            )
            """,
            
            'user_dim': f"""
            CREATE TABLE IF NOT EXISTS {schema_name}.user_dim (
                user_id VARCHAR(255) PRIMARY KEY,
                username VARCHAR(150),
                email VARCHAR(255),
                first_name VARCHAR(100),
                last_name VARCHAR(100),
                registration_date DATE,
                is_active BOOLEAN DEFAULT TRUE,
                last_activity_date DATE,
                total_events INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
            """,
            
            'daily_metrics': f"""
            CREATE TABLE IF NOT EXISTS {schema_name}.daily_metrics (
                metric_date DATE PRIMARY KEY,
                total_events INTEGER DEFAULT 0,
                unique_users INTEGER DEFAULT 0,
                new_registrations INTEGER DEFAULT 0,
                active_users INTEGER DEFAULT 0,
                events_by_type JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            """
        }
        
        results = {}
        with engine.connect() as conn:
            for table_name, create_sql in tables.items():
                try:
                    conn.execute(text(create_sql))
                    results[table_name] = "Created successfully"
                except Exception as e:
                    results[table_name] = f"Error: {str(e)}"
            conn.commit()
        
        return results

class ETLPipeline:
    """ETL pipeline for processing event data."""
    
    def __init__(self, analyzer: EventStreamAnalyzer, exporter: DataExporter):
        self.analyzer = analyzer
        self.exporter = exporter
        self.pipeline_state = {}
    
    async def run_daily_etl(
        self,
        target_date: datetime,
        output_dir: str = "./data_exports"
    ) -> Dict[str, str]:
        """Run daily ETL process."""
        
        output_path = Path(output_dir)
        output_path.mkdir(exist_ok=True)
        
        start_date = target_date.replace(hour=0, minute=0, second=0, microsecond=0)
        end_date = start_date.replace(hour=23, minute=59, second=59)
        
        date_str = target_date.strftime('%Y-%m-%d')
        
        results = {}
        
        # 1. Extract raw events
        csv_path = output_path / f"events_{date_str}.csv"
        results['csv_export'] = await self.exporter.export_to_csv(
            str(csv_path),
            start_date=start_date,
            end_date=end_date
        )
        
        # 2. Create analytics report
        excel_path = output_path / f"analytics_{date_str}.xlsx"
        results['excel_export'] = await self.exporter.export_to_excel(
            str(excel_path),
            include_analytics=True,
            start_date=start_date,
            end_date=end_date
        )
        
        # 3. Archive to Parquet
        parquet_path = output_path / f"events_{date_str}.parquet"
        results['parquet_export'] = await self.exporter.export_to_parquet(
            str(parquet_path),
            start_date=start_date,
            end_date=end_date
        )
        
        return results
    
    async def run_incremental_etl(
        self,
        last_processed_position: int,
        batch_size: int = 10000
    ) -> Dict[str, Any]:
        """Run incremental ETL from last processed position."""
        
        # This would need integration with the event store to get events
        # from a specific position - simplified implementation
        
        results = {
            'start_position': last_processed_position,
            'processed_events': 0,
            'end_position': last_processed_position,
            'errors': []
        }
        
        # In a real implementation, you would:
        # 1. Get events from last_processed_position
        # 2. Process them in batches
        # 3. Update data warehouse tables
        # 4. Update last_processed_position
        
        return results
    
    async def schedule_etl_pipeline(self, interval_hours: int = 24):
        """Schedule ETL pipeline to run at regular intervals."""
        
        while True:
            try:
                current_time = datetime.now()
                print(f"Running ETL pipeline at {current_time}")
                
                # Run daily ETL for yesterday
                yesterday = current_time - pd.Timedelta(days=1)
                results = await self.run_daily_etl(yesterday)
                
                print(f"ETL completed: {results}")
                
                # Wait for next interval
                await asyncio.sleep(interval_hours * 3600)
                
            except Exception as e:
                print(f"ETL pipeline error: {e}")
                await asyncio.sleep(3600)  # Wait 1 hour before retry
```

## 🔗 Example Usage

Create `examples/pandas_analytics_example.py`:

```python
import asyncio
import pandas as pd
from datetime import datetime, timedelta

from eventuali import EventStore, EventStreamer
from eventuali.streaming import Subscription

# Import our analytics modules
from analytics.stream_converter import EventStreamAnalyzer, RealTimeAnalytics
from analytics.user_analytics import UserAnalytics, visualize_user_analytics
from analytics.dashboard import RealTimeDashboard
from analytics.data_export import DataExporter, ETLPipeline

async def pandas_analytics_example():
    """Comprehensive example of Pandas integration with Eventuali."""
    
    print("🐼 Eventuali + Pandas Analytics Example")
    print("=" * 50)
    
    # Setup
    event_store = await EventStore.create("sqlite://:memory:")
    event_streamer = EventStreamer(capacity=5000)
    
    # Create analyzer and related components
    analyzer = EventStreamAnalyzer(event_store, event_streamer)
    real_time_analytics = RealTimeAnalytics(event_streamer)
    dashboard = RealTimeDashboard(real_time_analytics)
    exporter = DataExporter(analyzer)
    etl_pipeline = ETLPipeline(analyzer, exporter)
    
    print("✅ Components initialized")
    
    # Example 1: Stream Analysis
    print("\n📊 Example 1: Real-time Stream Analysis")
    
    # Create subscription for all user events
    user_subscription = Subscription(
        id="pandas-analytics",
        aggregate_type_filter="User"
    )
    
    # Start real-time analytics (would run in background)
    print("Starting real-time analytics...")
    
    # Simulate some events (in real scenario, these come from event store)
    print("Simulating events...")
    
    # Example 2: Historical Analysis
    print("\n📈 Example 2: Historical Data Analysis")
    
    # In a real scenario with historical data:
    # events_df = await analyzer.events_to_dataframe(
    #     start_date=datetime.now() - timedelta(days=30),
    #     aggregate_type="User"
    # )
    
    # For demo, create sample data
    sample_events = pd.DataFrame([
        {
            'aggregate_id': 'user-1',
            'aggregate_type': 'User',
            'event_type': 'UserRegistered',
            'timestamp': datetime.now() - timedelta(days=10),
            'aggregate_version': 1,
            'data_username': 'alice',
            'data_email': 'alice@example.com'
        },
        {
            'aggregate_id': 'user-2', 
            'aggregate_type': 'User',
            'event_type': 'UserRegistered',
            'timestamp': datetime.now() - timedelta(days=8),
            'aggregate_version': 1,
            'data_username': 'bob',
            'data_email': 'bob@example.com'
        },
        {
            'aggregate_id': 'user-1',
            'aggregate_type': 'User', 
            'event_type': 'UserEmailChanged',
            'timestamp': datetime.now() - timedelta(days=5),
            'aggregate_version': 2,
            'data_old_email': 'alice@example.com',
            'data_new_email': 'alice.smith@example.com'
        }
    ])
    
    print(f"Sample dataset: {len(sample_events)} events")
    print(sample_events[['event_type', 'timestamp', 'aggregate_id']].head())
    
    # Perform analytics
    analytics = UserAnalytics(sample_events)
    
    # Registration analysis
    reg_analysis = analytics.user_registration_analysis()
    print(f"\n📅 Registration Analysis:")
    for name, data in reg_analysis.items():
        print(f"  {name}: {len(data)} records")
    
    # Activity analysis
    activity_analysis = analytics.user_activity_analysis()
    print(f"\n🎯 Activity Analysis:")
    for name, data in activity_analysis.items():
        print(f"  {name}: {len(data)} records")
    
    # Journey analysis
    journey_analysis = analytics.user_journey_analysis()
    print(f"\n🛤️  Journey Analysis:")
    for name, data in journey_analysis.items():
        print(f"  {name}: {len(data)} records")
    
    # Example 3: Data Export
    print("\n💾 Example 3: Data Export")
    
    # Export sample data
    import tempfile
    with tempfile.TemporaryDirectory() as temp_dir:
        
        # CSV export
        csv_path = f"{temp_dir}/sample_events.csv"
        sample_events.to_csv(csv_path, index=False)
        print(f"✅ Exported to CSV: {csv_path}")
        
        # Excel export with analytics
        excel_path = f"{temp_dir}/analytics_report.xlsx"
        with pd.ExcelWriter(excel_path) as writer:
            sample_events.to_excel(writer, sheet_name='Events', index=False)
            
            # Add analytics sheets
            for name, data in reg_analysis.items():
                data.to_excel(writer, sheet_name=f'Reg_{name[:20]}', index=False)
        
        print(f"✅ Exported to Excel: {excel_path}")
        
        # Parquet export
        parquet_path = f"{temp_dir}/events.parquet"
        sample_events.to_parquet(parquet_path, index=False)
        print(f"✅ Exported to Parquet: {parquet_path}")
    
    # Example 4: Real-time Dashboard Metrics
    print("\n📊 Example 4: Dashboard Metrics")
    
    # Simulate dashboard data
    dashboard_data = dashboard.create_dashboard()
    current_metrics = dashboard_data.get('current_metrics', {})
    
    print("Dashboard Metrics:")
    for key, value in current_metrics.items():
        print(f"  {key}: {value}")
    
    print("\n🎉 Pandas Analytics Example Complete!")
    print("=" * 50)

if __name__ == "__main__":
    asyncio.run(pandas_analytics_example())
```

## 🔗 Related Documentation

- **[FastAPI Integration](fastapi-integration.md)** - REST API patterns
- **[Performance Guide](../performance/README.md)** - Optimization strategies  
- **[Streaming API](../api/streaming/README.md)** - Event streaming reference
- **[Examples](../../examples/10_analytics_pandas.py)** - Complete Pandas examples

---

**Next**: Try the [Microservices Integration Guide](microservices-integration.md) or explore [deployment patterns](../deployment/README.md).