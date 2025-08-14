# Django Integration Guide

**Building Django applications with Eventuali event sourcing**

This guide demonstrates how to integrate Eventuali with Django for building event-sourced web applications with Django's ORM, views, and admin interface.

## 🎯 What You'll Learn

- ✅ Django settings and configuration for Eventuali
- ✅ Model integration with event sourcing
- ✅ Django views with event store operations
- ✅ Admin interface for event management
- ✅ Django REST Framework integration
- ✅ Background task processing with Celery
- ✅ Testing patterns for event-sourced Django apps

## 📋 Prerequisites

```bash
# Install dependencies
uv add django eventuali
uv add djangorestframework  # For REST API
uv add celery redis  # For background tasks
uv add django-extensions  # For management commands
```

## 🚀 Project Setup

### Django Settings Configuration

Create `myproject/settings.py`:

```python
import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent

# Django core settings
SECRET_KEY = os.getenv('SECRET_KEY', 'your-secret-key-here')
DEBUG = os.getenv('DEBUG', 'False').lower() == 'true'
ALLOWED_HOSTS = os.getenv('ALLOWED_HOSTS', 'localhost,127.0.0.1').split(',')

# Application definition
INSTALLED_APPS = [
    'django.contrib.admin',
    'django.contrib.auth',
    'django.contrib.contenttypes',
    'django.contrib.sessions',
    'django.contrib.messages',
    'django.contrib.staticfiles',
    
    # Third party apps
    'rest_framework',
    'django_extensions',
    
    # Local apps
    'users',
    'orders',
    'eventuali_integration',
]

MIDDLEWARE = [
    'django.middleware.security.SecurityMiddleware',
    'django.contrib.sessions.middleware.SessionMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.middleware.csrf.CsrfViewMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
    'django.contrib.messages.middleware.MessageMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',
    'eventuali_integration.middleware.EventStoreMiddleware',  # Custom middleware
]

ROOT_URLCONF = 'myproject.urls'

# Database configuration
DATABASES = {
    'default': {
        'ENGINE': 'django.db.backends.postgresql',
        'NAME': os.getenv('DB_NAME', 'django_eventuali'),
        'USER': os.getenv('DB_USER', 'postgres'),
        'PASSWORD': os.getenv('DB_PASSWORD', 'password'),
        'HOST': os.getenv('DB_HOST', 'localhost'),
        'PORT': os.getenv('DB_PORT', '5432'),
    }
}

# Eventuali configuration
EVENTUALI_CONFIG = {
    'CONNECTION_STRING': os.getenv(
        'EVENTUALI_CONNECTION_STRING',
        f"postgresql://{DATABASES['default']['USER']}:"
        f"{DATABASES['default']['PASSWORD']}@"
        f"{DATABASES['default']['HOST']}:"
        f"{DATABASES['default']['PORT']}/"
        f"{DATABASES['default']['NAME']}"
    ),
    'STREAM_CAPACITY': int(os.getenv('EVENTUALI_STREAM_CAPACITY', '5000')),
    'AUTO_REGISTER_EVENTS': True,
    'ENABLE_PROJECTIONS': True,
}

# Celery configuration for background tasks
CELERY_BROKER_URL = os.getenv('REDIS_URL', 'redis://localhost:6379/0')
CELERY_RESULT_BACKEND = os.getenv('REDIS_URL', 'redis://localhost:6379/0')
CELERY_ACCEPT_CONTENT = ['json']
CELERY_TASK_SERIALIZER = 'json'
CELERY_RESULT_SERIALIZER = 'json'
CELERY_TIMEZONE = 'UTC'

# REST Framework configuration
REST_FRAMEWORK = {
    'DEFAULT_AUTHENTICATION_CLASSES': [
        'rest_framework.authentication.SessionAuthentication',
        'rest_framework.authentication.TokenAuthentication',
    ],
    'DEFAULT_PERMISSION_CLASSES': [
        'rest_framework.permissions.IsAuthenticated',
    ],
    'DEFAULT_PAGINATION_CLASS': 'rest_framework.pagination.PageNumberPagination',
    'PAGE_SIZE': 20,
}

# Logging configuration
LOGGING = {
    'version': 1,
    'disable_existing_loggers': False,
    'handlers': {
        'console': {
            'class': 'logging.StreamHandler',
        },
    },
    'loggers': {
        'eventuali': {
            'handlers': ['console'],
            'level': 'INFO',
        },
    },
}
```

## 🔧 Eventuali Integration App

Create `eventuali_integration/apps.py`:

```python
from django.apps import AppConfig
import asyncio
import logging

logger = logging.getLogger(__name__)

class EventualiIntegrationConfig(AppConfig):
    default_auto_field = 'django.db.models.BigAutoField'
    name = 'eventuali_integration'
    
    def ready(self):
        """Initialize Eventuali components when Django starts."""
        from .services import EventStoreService
        
        # Initialize event store service
        try:
            # Start async components in background
            import threading
            
            def start_eventuali():
                try:
                    loop = asyncio.new_event_loop()
                    asyncio.set_event_loop(loop)
                    loop.run_until_complete(EventStoreService.initialize())
                    logger.info("Eventuali integration initialized successfully")
                except Exception as e:
                    logger.error(f"Failed to initialize Eventuali: {e}")
            
            thread = threading.Thread(target=start_eventuali, daemon=True)
            thread.start()
            
        except Exception as e:
            logger.error(f"Error during Eventuali initialization: {e}")
```

### Event Store Service

Create `eventuali_integration/services.py`:

```python
import asyncio
from typing import Optional, Dict, Any, Type
from django.conf import settings
from eventuali import EventStore, EventStreamer
from eventuali.streaming import Subscription
from eventuali.exceptions import EventualiError
import logging

logger = logging.getLogger(__name__)

class EventStoreService:
    """Singleton service for managing EventStore and EventStreamer."""
    
    _instance: Optional['EventStoreService'] = None
    _event_store: Optional[EventStore] = None
    _event_streamer: Optional[EventStreamer] = None
    _lock = asyncio.Lock()
    
    def __new__(cls) -> 'EventStoreService':
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance
    
    @classmethod
    async def initialize(cls) -> 'EventStoreService':
        """Initialize the event store service."""
        service = cls()
        
        async with cls._lock:
            if cls._event_store is None:
                config = settings.EVENTUALI_CONFIG
                
                # Create event store
                cls._event_store = await EventStore.create(
                    config['CONNECTION_STRING']
                )
                
                # Create event streamer
                cls._event_streamer = EventStreamer(
                    capacity=config['STREAM_CAPACITY']
                )
                
                # Auto-register events if enabled
                if config.get('AUTO_REGISTER_EVENTS', True):
                    await service._register_domain_events()
                
                logger.info("EventStore and EventStreamer initialized")
        
        return service
    
    @classmethod
    async def get_event_store(cls) -> EventStore:
        """Get the EventStore instance."""
        if cls._event_store is None:
            await cls.initialize()
        return cls._event_store
    
    @classmethod
    async def get_event_streamer(cls) -> EventStreamer:
        """Get the EventStreamer instance."""
        if cls._event_streamer is None:
            await cls.initialize()
        return cls._event_streamer
    
    async def _register_domain_events(self):
        """Auto-register domain events from Django apps."""
        from django.apps import apps
        
        for app_config in apps.get_app_configs():
            try:
                # Try to import events module from each app
                events_module = app_config.module.__name__ + '.events'
                module = __import__(events_module, fromlist=[''])
                
                # Register event classes
                for attr_name in dir(module):
                    attr = getattr(module, attr_name)
                    if (isinstance(attr, type) and 
                        hasattr(attr, '__bases__') and 
                        any('Event' in base.__name__ for base in attr.__bases__)):
                        
                        event_type = attr.__name__
                        self._event_store.register_event_class(event_type, attr)
                        logger.info(f"Registered event class: {event_type}")
                        
            except (ImportError, AttributeError):
                # App doesn't have events module - skip
                continue

# Convenience functions for Django views
async def get_event_store() -> EventStore:
    """Get EventStore instance for use in views."""
    return await EventStoreService.get_event_store()

async def get_event_streamer() -> EventStreamer:
    """Get EventStreamer instance for use in views."""
    return await EventStoreService.get_event_streamer()
```

### Custom Middleware

Create `eventuali_integration/middleware.py`:

```python
import asyncio
from django.utils.deprecation import MiddlewareMixin
from .services import EventStoreService

class EventStoreMiddleware(MiddlewareMixin):
    """Middleware to ensure EventStore is available in requests."""
    
    def __init__(self, get_response):
        self.get_response = get_response
        super().__init__(get_response)
    
    def process_request(self, request):
        """Add event store service to request."""
        # Ensure service is initialized
        try:
            if hasattr(request, '_event_store_service'):
                return
            
            # Create new event loop for this request if needed
            try:
                loop = asyncio.get_event_loop()
            except RuntimeError:
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)
            
            request._event_store_service = EventStoreService()
            
        except Exception as e:
            # Log error but don't break the request
            import logging
            logger = logging.getLogger(__name__)
            logger.error(f"Error in EventStoreMiddleware: {e}")
```

## 👥 User Management with Django

### Domain Events

Create `users/events.py`:

```python
from eventuali import Event
from datetime import datetime
from typing import Optional
from pydantic import EmailStr, Field

class UserRegistered(Event):
    """User registration event."""
    username: str = Field(..., min_length=3, max_length=150)
    email: EmailStr
    first_name: Optional[str] = None
    last_name: Optional[str] = None

class UserEmailChanged(Event):
    """User email change event."""
    old_email: EmailStr
    new_email: EmailStr

class UserProfileUpdated(Event):
    """User profile update event."""
    first_name: Optional[str] = None
    last_name: Optional[str] = None
    phone: Optional[str] = None

class UserDeactivated(Event):
    """User account deactivation event."""
    reason: str
    deactivated_by: str  # User ID of who deactivated
```

### Domain Aggregate

Create `users/aggregates.py`:

```python
from eventuali import Aggregate
from typing import Optional
from datetime import datetime
from .events import UserRegistered, UserEmailChanged, UserProfileUpdated, UserDeactivated

class User(Aggregate):
    """User aggregate representing user account lifecycle."""
    
    def __init__(self, id: str, version: int = 0):
        super().__init__(id, version)
        self.username: Optional[str] = None
        self.email: Optional[str] = None
        self.first_name: Optional[str] = None
        self.last_name: Optional[str] = None
        self.phone: Optional[str] = None
        self.is_active: bool = False
        self.registered_at: Optional[datetime] = None
        self.deactivated_at: Optional[datetime] = None
        self.deactivation_reason: Optional[str] = None
    
    def register(self, username: str, email: str, first_name: str = None, last_name: str = None):
        """Register a new user."""
        if self.version > 0:
            raise ValueError("User already registered")
        
        event = UserRegistered(
            username=username,
            email=email,
            first_name=first_name,
            last_name=last_name
        )
        self.apply(event)
    
    def change_email(self, new_email: str):
        """Change user's email address."""
        if not self.is_active:
            raise ValueError("Cannot change email for inactive user")
        if new_email == self.email:
            raise ValueError("New email is the same as current email")
        
        event = UserEmailChanged(
            old_email=self.email,
            new_email=new_email
        )
        self.apply(event)
    
    def update_profile(self, first_name: str = None, last_name: str = None, phone: str = None):
        """Update user profile information."""
        if not self.is_active:
            raise ValueError("Cannot update profile for inactive user")
        
        event = UserProfileUpdated(
            first_name=first_name,
            last_name=last_name,
            phone=phone
        )
        self.apply(event)
    
    def deactivate(self, reason: str, deactivated_by: str):
        """Deactivate user account."""
        if not self.is_active:
            raise ValueError("User already deactivated")
        
        event = UserDeactivated(
            reason=reason,
            deactivated_by=deactivated_by
        )
        self.apply(event)
    
    # Event handlers
    def apply_userregistered(self, event: UserRegistered):
        """Apply user registration event."""
        self.username = event.username
        self.email = event.email
        self.first_name = event.first_name
        self.last_name = event.last_name
        self.is_active = True
        self.registered_at = event.timestamp
    
    def apply_useremailchanged(self, event: UserEmailChanged):
        """Apply email change event."""
        self.email = event.new_email
    
    def apply_userprofileupdated(self, event: UserProfileUpdated):
        """Apply profile update event."""
        if event.first_name is not None:
            self.first_name = event.first_name
        if event.last_name is not None:
            self.last_name = event.last_name
        if event.phone is not None:
            self.phone = event.phone
    
    def apply_userdeactivated(self, event: UserDeactivated):
        """Apply user deactivation event."""
        self.is_active = False
        self.deactivated_at = event.timestamp
        self.deactivation_reason = event.reason
    
    @property
    def full_name(self) -> str:
        """Get user's full name."""
        if self.first_name and self.last_name:
            return f"{self.first_name} {self.last_name}"
        return self.first_name or self.last_name or self.username
```

## 🌐 Django Views Integration

### Function-Based Views

Create `users/views.py`:

```python
import asyncio
from django.shortcuts import render, redirect
from django.contrib import messages
from django.contrib.auth.decorators import login_required
from django.views.decorators.http import require_http_methods
from django.http import JsonResponse, Http404
from django.core.paginator import Paginator
from uuid import uuid4

from eventuali_integration.services import get_event_store
from .aggregates import User
from .forms import UserRegistrationForm, UserProfileForm
from .models import UserProjection  # Read model

@require_http_methods(["GET", "POST"])
async def register_user(request):
    """User registration view."""
    if request.method == 'POST':
        form = UserRegistrationForm(request.POST)
        if form.is_valid():
            try:
                # Create user aggregate
                user_id = str(uuid4())
                user = User(id=user_id)
                
                # Apply registration
                user.register(
                    username=form.cleaned_data['username'],
                    email=form.cleaned_data['email'],
                    first_name=form.cleaned_data.get('first_name'),
                    last_name=form.cleaned_data.get('last_name')
                )
                
                # Save to event store
                event_store = await get_event_store()
                await event_store.save(user)
                user.mark_events_as_committed()
                
                messages.success(request, 'User registered successfully!')
                return redirect('user_detail', user_id=user_id)
                
            except Exception as e:
                messages.error(request, f'Registration failed: {str(e)}')
    else:
        form = UserRegistrationForm()
    
    return render(request, 'users/register.html', {'form': form})

@login_required
async def user_detail(request, user_id):
    """User detail view."""
    try:
        event_store = await get_event_store()
        user = await event_store.load(User, user_id)
        
        if not user:
            raise Http404("User not found")
        
        return render(request, 'users/detail.html', {'user': user})
        
    except Exception as e:
        messages.error(request, f'Error loading user: {str(e)}')
        return redirect('user_list')

@login_required
async def user_list(request):
    """User list view using read model."""
    # Use projection/read model for efficient queries
    users = UserProjection.objects.filter(is_active=True).order_by('-created_at')
    
    paginator = Paginator(users, 25)
    page_number = request.GET.get('page')
    page_obj = paginator.get_page(page_number)
    
    return render(request, 'users/list.html', {'page_obj': page_obj})

@login_required
async def update_user_profile(request, user_id):
    """Update user profile."""
    if request.method == 'POST':
        form = UserProfileForm(request.POST)
        if form.is_valid():
            try:
                # Load aggregate
                event_store = await get_event_store()
                user = await event_store.load(User, user_id)
                
                if not user:
                    raise Http404("User not found")
                
                # Update profile
                user.update_profile(
                    first_name=form.cleaned_data.get('first_name'),
                    last_name=form.cleaned_data.get('last_name'),
                    phone=form.cleaned_data.get('phone')
                )
                
                # Save changes
                await event_store.save(user)
                user.mark_events_as_committed()
                
                messages.success(request, 'Profile updated successfully!')
                return redirect('user_detail', user_id=user_id)
                
            except Exception as e:
                messages.error(request, f'Update failed: {str(e)}')
    else:
        # Load user for form initialization
        try:
            event_store = await get_event_store()
            user = await event_store.load(User, user_id)
            
            if not user:
                raise Http404("User not found")
            
            form = UserProfileForm(initial={
                'first_name': user.first_name,
                'last_name': user.last_name,
                'phone': user.phone
            })
        except Exception as e:
            messages.error(request, f'Error loading user: {str(e)}')
            return redirect('user_list')
    
    return render(request, 'users/update_profile.html', {
        'form': form, 
        'user': user
    })

@login_required
async def user_events_api(request, user_id):
    """API endpoint to get user event history."""
    try:
        event_store = await get_event_store()
        events = await event_store.load_events(user_id)
        
        events_data = [
            {
                'event_type': event.event_type,
                'timestamp': event.timestamp.isoformat() if event.timestamp else None,
                'version': event.aggregate_version,
                'data': event.to_dict()
            }
            for event in events
        ]
        
        return JsonResponse({'events': events_data})
        
    except Exception as e:
        return JsonResponse({'error': str(e)}, status=500)
```

### Class-Based Views

Create `users/class_views.py`:

```python
import asyncio
from django.views.generic import View, TemplateView
from django.contrib.auth.mixins import LoginRequiredMixin
from django.http import JsonResponse, Http404
from django.shortcuts import render, redirect
from django.contrib import messages
from uuid import uuid4

from eventuali_integration.services import get_event_store
from .aggregates import User
from .forms import UserRegistrationForm

class AsyncViewMixin:
    """Mixin to handle async operations in Django views."""
    
    def dispatch(self, request, *args, **kwargs):
        """Handle async view methods."""
        if asyncio.iscoroutinefunction(self.get):
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            try:
                return loop.run_until_complete(self.get(request, *args, **kwargs))
            finally:
                loop.close()
        return super().dispatch(request, *args, **kwargs)

class UserRegistrationView(AsyncViewMixin, View):
    """Class-based user registration view."""
    
    template_name = 'users/register.html'
    form_class = UserRegistrationForm
    
    async def get(self, request):
        form = self.form_class()
        return render(request, self.template_name, {'form': form})
    
    async def post(self, request):
        form = self.form_class(request.POST)
        
        if form.is_valid():
            try:
                # Create and register user
                user_id = str(uuid4())
                user = User(id=user_id)
                
                user.register(
                    username=form.cleaned_data['username'],
                    email=form.cleaned_data['email'],
                    first_name=form.cleaned_data.get('first_name'),
                    last_name=form.cleaned_data.get('last_name')
                )
                
                # Save to event store
                event_store = await get_event_store()
                await event_store.save(user)
                user.mark_events_as_committed()
                
                messages.success(request, 'User registered successfully!')
                return redirect('user_detail', user_id=user_id)
                
            except Exception as e:
                messages.error(request, f'Registration failed: {str(e)}')
        
        return render(request, self.template_name, {'form': form})

class UserDetailView(LoginRequiredMixin, AsyncViewMixin, View):
    """Class-based user detail view."""
    
    template_name = 'users/detail.html'
    
    async def get(self, request, user_id):
        try:
            event_store = await get_event_store()
            user = await event_store.load(User, user_id)
            
            if not user:
                raise Http404("User not found")
            
            # Load recent events for activity feed
            events = await event_store.load_events(user_id)
            recent_events = events[-5:]  # Last 5 events
            
            return render(request, self.template_name, {
                'user': user,
                'recent_events': recent_events
            })
            
        except Exception as e:
            messages.error(request, f'Error loading user: {str(e)}')
            return redirect('user_list')

class UserEventsAPIView(LoginRequiredMixin, AsyncViewMixin, View):
    """API view for user events."""
    
    async def get(self, request, user_id):
        try:
            event_store = await get_event_store()
            
            # Check if user exists
            user = await event_store.load(User, user_id)
            if not user:
                return JsonResponse({'error': 'User not found'}, status=404)
            
            # Load events
            events = await event_store.load_events(user_id)
            
            # Pagination
            page = int(request.GET.get('page', 1))
            page_size = int(request.GET.get('page_size', 20))
            start = (page - 1) * page_size
            end = start + page_size
            
            paginated_events = events[start:end]
            
            events_data = [
                {
                    'id': f"{event.aggregate_id}-{event.aggregate_version}",
                    'event_type': event.event_type,
                    'timestamp': event.timestamp.isoformat() if event.timestamp else None,
                    'version': event.aggregate_version,
                    'data': {
                        key: value for key, value in event.to_dict().items()
                        if key not in ['aggregate_id', 'aggregate_type', 'aggregate_version']
                    }
                }
                for event in paginated_events
            ]
            
            return JsonResponse({
                'events': events_data,
                'pagination': {
                    'page': page,
                    'page_size': page_size,
                    'total_events': len(events),
                    'has_next': end < len(events),
                    'has_previous': page > 1
                }
            })
            
        except Exception as e:
            return JsonResponse({'error': str(e)}, status=500)
```

## 📋 Django Forms

Create `users/forms.py`:

```python
from django import forms
from django.core.exceptions import ValidationError
from django.contrib.auth.models import User as DjangoUser

class UserRegistrationForm(forms.Form):
    """Form for user registration."""
    
    username = forms.CharField(
        max_length=150,
        widget=forms.TextInput(attrs={
            'class': 'form-control',
            'placeholder': 'Username'
        })
    )
    
    email = forms.EmailField(
        widget=forms.EmailInput(attrs={
            'class': 'form-control',
            'placeholder': 'Email address'
        })
    )
    
    first_name = forms.CharField(
        max_length=30,
        required=False,
        widget=forms.TextInput(attrs={
            'class': 'form-control',
            'placeholder': 'First name'
        })
    )
    
    last_name = forms.CharField(
        max_length=30,
        required=False,
        widget=forms.TextInput(attrs={
            'class': 'form-control',
            'placeholder': 'Last name'
        })
    )
    
    def clean_username(self):
        """Validate username uniqueness."""
        username = self.cleaned_data['username']
        
        # Check Django User model for uniqueness
        if DjangoUser.objects.filter(username=username).exists():
            raise ValidationError("Username already taken")
        
        # Additional validation
        if len(username) < 3:
            raise ValidationError("Username must be at least 3 characters long")
        
        return username
    
    def clean_email(self):
        """Validate email uniqueness."""
        email = self.cleaned_data['email']
        
        # Check Django User model for uniqueness
        if DjangoUser.objects.filter(email=email).exists():
            raise ValidationError("Email already registered")
        
        return email

class UserProfileForm(forms.Form):
    """Form for updating user profile."""
    
    first_name = forms.CharField(
        max_length=30,
        required=False,
        widget=forms.TextInput(attrs={
            'class': 'form-control',
            'placeholder': 'First name'
        })
    )
    
    last_name = forms.CharField(
        max_length=30,
        required=False,
        widget=forms.TextInput(attrs={
            'class': 'form-control',
            'placeholder': 'Last name'
        })
    )
    
    phone = forms.CharField(
        max_length=20,
        required=False,
        widget=forms.TextInput(attrs={
            'class': 'form-control',
            'placeholder': 'Phone number'
        })
    )
    
    def clean_phone(self):
        """Validate phone number format."""
        phone = self.cleaned_data.get('phone')
        if phone:
            # Basic phone validation
            import re
            if not re.match(r'^[\+]?[1-9][\d]{0,15}$', phone.replace(' ', '').replace('-', '')):
                raise ValidationError("Invalid phone number format")
        return phone

class UserEmailChangeForm(forms.Form):
    """Form for changing user email."""
    
    new_email = forms.EmailField(
        widget=forms.EmailInput(attrs={
            'class': 'form-control',
            'placeholder': 'New email address'
        })
    )
    
    confirm_email = forms.EmailField(
        widget=forms.EmailInput(attrs={
            'class': 'form-control',
            'placeholder': 'Confirm new email'
        })
    )
    
    def clean(self):
        """Validate email confirmation."""
        cleaned_data = super().clean()
        new_email = cleaned_data.get('new_email')
        confirm_email = cleaned_data.get('confirm_email')
        
        if new_email and confirm_email and new_email != confirm_email:
            raise ValidationError("Email addresses don't match")
        
        return cleaned_data
    
    def clean_new_email(self):
        """Validate new email uniqueness."""
        email = self.cleaned_data['new_email']
        
        # Check Django User model for uniqueness
        if DjangoUser.objects.filter(email=email).exists():
            raise ValidationError("Email already registered")
        
        return email
```

## 📊 Read Models (Projections)

Create `users/models.py`:

```python
from django.db import models
from django.utils import timezone

class UserProjection(models.Model):
    """Read model for user data optimized for queries."""
    
    user_id = models.CharField(max_length=255, unique=True, db_index=True)
    username = models.CharField(max_length=150, unique=True, db_index=True)
    email = models.EmailField(db_index=True)
    first_name = models.CharField(max_length=30, blank=True)
    last_name = models.CharField(max_length=30, blank=True)
    phone = models.CharField(max_length=20, blank=True)
    is_active = models.BooleanField(default=True, db_index=True)
    
    # Timestamps
    created_at = models.DateTimeField(db_index=True)
    updated_at = models.DateTimeField(auto_now=True)
    deactivated_at = models.DateTimeField(null=True, blank=True)
    
    # Metadata
    current_version = models.PositiveIntegerField(default=0)
    last_event_processed = models.PositiveIntegerField(default=0)
    
    class Meta:
        db_table = 'user_projection'
        indexes = [
            models.Index(fields=['is_active', 'created_at']),
            models.Index(fields=['email', 'is_active']),
            models.Index(fields=['username', 'is_active']),
        ]
        
    def __str__(self):
        return f"{self.username} ({self.email})"
    
    @property
    def full_name(self):
        """Get user's full name."""
        if self.first_name and self.last_name:
            return f"{self.first_name} {self.last_name}"
        return self.first_name or self.last_name or self.username

class UserEventLog(models.Model):
    """Log of user events for audit trail."""
    
    user_id = models.CharField(max_length=255, db_index=True)
    event_type = models.CharField(max_length=100, db_index=True)
    event_version = models.PositiveIntegerField()
    event_data = models.JSONField()
    timestamp = models.DateTimeField(db_index=True)
    
    # Metadata
    processed_at = models.DateTimeField(auto_now_add=True)
    global_position = models.BigIntegerField(null=True, blank=True)
    
    class Meta:
        db_table = 'user_event_log'
        indexes = [
            models.Index(fields=['user_id', 'event_version']),
            models.Index(fields=['event_type', 'timestamp']),
            models.Index(fields=['timestamp']),
        ]
        ordering = ['event_version']
    
    def __str__(self):
        return f"{self.event_type} for {self.user_id} v{self.event_version}"
```

## 🔄 Event Projections with Celery

Create `users/projections.py`:

```python
import asyncio
from typing import Dict, Any
from django.utils import timezone
from eventuali import Event
from eventuali.streaming import Projection
from .models import UserProjection, UserEventLog
from .events import UserRegistered, UserEmailChanged, UserProfileUpdated, UserDeactivated

class UserProjectionHandler(Projection):
    """Projection handler for maintaining user read models."""
    
    def __init__(self):
        self.last_processed_position = 0
    
    async def handle_event(self, event: Event) -> None:
        """Process event and update read models."""
        try:
            # Route to specific handler
            handler_name = f"handle_{event.event_type.lower()}"
            handler = getattr(self, handler_name, None)
            
            if handler:
                await handler(event)
            
            # Update event log
            await self._log_event(event)
            
        except Exception as e:
            # Log error but don't stop processing
            import logging
            logger = logging.getLogger(__name__)
            logger.error(f"Error processing event {event.event_type}: {e}")
            raise
    
    async def handle_userregistered(self, event: UserRegistered) -> None:
        """Handle user registration event."""
        from django.db import transaction
        
        with transaction.atomic():
            user_projection, created = UserProjection.objects.get_or_create(
                user_id=event.aggregate_id,
                defaults={
                    'username': event.username,
                    'email': event.email,
                    'first_name': event.first_name or '',
                    'last_name': event.last_name or '',
                    'is_active': True,
                    'created_at': event.timestamp or timezone.now(),
                    'current_version': event.aggregate_version,
                }
            )
            
            if not created:
                # Update existing (shouldn't happen for registration)
                user_projection.username = event.username
                user_projection.email = event.email
                user_projection.first_name = event.first_name or ''
                user_projection.last_name = event.last_name or ''
                user_projection.current_version = event.aggregate_version
                user_projection.save()
    
    async def handle_useremailchanged(self, event: UserEmailChanged) -> None:
        """Handle email change event."""
        from django.db import transaction
        
        with transaction.atomic():
            try:
                user_projection = UserProjection.objects.get(user_id=event.aggregate_id)
                user_projection.email = event.new_email
                user_projection.current_version = event.aggregate_version
                user_projection.save()
            except UserProjection.DoesNotExist:
                # Log warning - projection should exist
                import logging
                logger = logging.getLogger(__name__)
                logger.warning(f"UserProjection not found for {event.aggregate_id}")
    
    async def handle_userprofileupdated(self, event: UserProfileUpdated) -> None:
        """Handle profile update event."""
        from django.db import transaction
        
        with transaction.atomic():
            try:
                user_projection = UserProjection.objects.get(user_id=event.aggregate_id)
                
                if event.first_name is not None:
                    user_projection.first_name = event.first_name
                if event.last_name is not None:
                    user_projection.last_name = event.last_name
                if event.phone is not None:
                    user_projection.phone = event.phone
                
                user_projection.current_version = event.aggregate_version
                user_projection.save()
                
            except UserProjection.DoesNotExist:
                import logging
                logger = logging.getLogger(__name__)
                logger.warning(f"UserProjection not found for {event.aggregate_id}")
    
    async def handle_userdeactivated(self, event: UserDeactivated) -> None:
        """Handle user deactivation event."""
        from django.db import transaction
        
        with transaction.atomic():
            try:
                user_projection = UserProjection.objects.get(user_id=event.aggregate_id)
                user_projection.is_active = False
                user_projection.deactivated_at = event.timestamp or timezone.now()
                user_projection.current_version = event.aggregate_version
                user_projection.save()
                
            except UserProjection.DoesNotExist:
                import logging
                logger = logging.getLogger(__name__)
                logger.warning(f"UserProjection not found for {event.aggregate_id}")
    
    async def _log_event(self, event: Event) -> None:
        """Log event for audit trail."""
        UserEventLog.objects.create(
            user_id=event.aggregate_id,
            event_type=event.event_type,
            event_version=event.aggregate_version,
            event_data=event.to_dict(),
            timestamp=event.timestamp or timezone.now(),
        )
    
    async def reset(self) -> None:
        """Reset projection to initial state."""
        UserProjection.objects.all().delete()
        UserEventLog.objects.all().delete()
        self.last_processed_position = 0
    
    async def get_last_processed_position(self) -> int:
        """Get last processed position."""
        return self.last_processed_position
    
    async def set_last_processed_position(self, position: int) -> None:
        """Set last processed position."""
        self.last_processed_position = position
```

## 🎯 Celery Tasks

Create `users/tasks.py`:

```python
import asyncio
from celery import shared_task
from django.conf import settings
from eventuali_integration.services import get_event_streamer
from eventuali.streaming import Subscription
from .projections import UserProjectionHandler
import logging

logger = logging.getLogger(__name__)

@shared_task
def start_user_projection_processor():
    """Start background task to process user events."""
    
    async def process_events():
        """Process user events continuously."""
        try:
            # Get event streamer
            streamer = await get_event_streamer()
            
            # Create subscription for user events
            subscription = Subscription(
                id="user-projection-processor",
                aggregate_type_filter="User"
            )
            
            # Subscribe to events
            receiver = await streamer.subscribe(subscription)
            
            # Create projection handler
            projection = UserProjectionHandler()
            
            logger.info("Started user projection processor")
            
            # Process events
            async for stream_event in receiver:
                try:
                    await projection.handle_event(stream_event.event)
                    await projection.set_last_processed_position(stream_event.global_position)
                    
                except Exception as e:
                    logger.error(f"Error processing user event: {e}")
                    # Continue processing other events
                    
        except Exception as e:
            logger.error(f"User projection processor failed: {e}")
            raise
    
    # Run async processing
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    
    try:
        loop.run_until_complete(process_events())
    finally:
        loop.close()

@shared_task
def send_welcome_email(user_id: str, email: str, name: str):
    """Send welcome email to new user."""
    # Implementation would integrate with email service
    logger.info(f"Sending welcome email to {name} <{email}> (User ID: {user_id})")
    
    # Example email service integration:
    # from django.core.mail import send_mail
    # send_mail(
    #     subject='Welcome to Our Platform!',
    #     message=f'Hello {name}, welcome to our platform!',
    #     from_email=settings.DEFAULT_FROM_EMAIL,
    #     recipient_list=[email],
    #     fail_silently=False,
    # )

@shared_task
def cleanup_old_event_logs():
    """Clean up old event logs (retention policy)."""
    from django.utils import timezone
    from datetime import timedelta
    from .models import UserEventLog
    
    # Delete logs older than 1 year
    cutoff_date = timezone.now() - timedelta(days=365)
    deleted_count, _ = UserEventLog.objects.filter(
        timestamp__lt=cutoff_date
    ).delete()
    
    logger.info(f"Cleaned up {deleted_count} old event log entries")
    return deleted_count

@shared_task
def rebuild_user_projections():
    """Rebuild all user projections from event store."""
    
    async def rebuild():
        """Rebuild projections asynchronously."""
        from eventuali_integration.services import get_event_store
        from .models import UserProjection
        
        # Clear existing projections
        UserProjection.objects.all().delete()
        
        # Get event store
        event_store = await get_event_store()
        
        # Create projection handler
        projection = UserProjectionHandler()
        
        # Get all user aggregates (this would need a different implementation in practice)
        # For now, assume we have a way to get all user IDs
        user_ids = UserProjection.objects.values_list('user_id', flat=True).distinct()
        
        for user_id in user_ids:
            try:
                # Load events for this user
                events = await event_store.load_events(user_id)
                
                # Replay events through projection
                for event in events:
                    await projection.handle_event(event)
                    
            except Exception as e:
                logger.error(f"Error rebuilding projection for user {user_id}: {e}")
        
        logger.info(f"Rebuilt projections for {len(user_ids)} users")
    
    # Run async rebuilding
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    
    try:
        loop.run_until_complete(rebuild())
    finally:
        loop.close()
```

## 🔗 URL Configuration

Create `users/urls.py`:

```python
from django.urls import path
from . import views, class_views

urlpatterns = [
    # Function-based views
    path('register/', views.register_user, name='user_register'),
    path('list/', views.user_list, name='user_list'),
    path('<str:user_id>/', views.user_detail, name='user_detail'),
    path('<str:user_id>/update/', views.update_user_profile, name='user_update'),
    path('<str:user_id>/events/', views.user_events_api, name='user_events_api'),
    
    # Class-based views
    path('cbv/register/', class_views.UserRegistrationView.as_view(), name='user_register_cbv'),
    path('cbv/<str:user_id>/', class_views.UserDetailView.as_view(), name='user_detail_cbv'),
    path('api/<str:user_id>/events/', class_views.UserEventsAPIView.as_view(), name='user_events_api_cbv'),
]
```

And in your main `myproject/urls.py`:

```python
from django.contrib import admin
from django.urls import path, include

urlpatterns = [
    path('admin/', admin.site.urls),
    path('users/', include('users.urls')),
    path('', include('users.urls')),  # Default to users app
]
```

## 🧪 Testing

Create `users/tests.py`:

```python
import asyncio
from django.test import TestCase, TransactionTestCase
from django.test.utils import override_settings
from unittest import mock
from uuid import uuid4

from eventuali import EventStore
from eventuali_integration.services import EventStoreService
from .aggregates import User
from .events import UserRegistered, UserEmailChanged
from .models import UserProjection
from .projections import UserProjectionHandler

class UserAggregateTests(TestCase):
    """Tests for User aggregate business logic."""
    
    def test_user_registration(self):
        """Test user registration creates correct event."""
        user = User(id='user-123')
        user.register('testuser', 'test@example.com', 'Test', 'User')
        
        # Check aggregate state
        self.assertEqual(user.username, 'testuser')
        self.assertEqual(user.email, 'test@example.com')
        self.assertEqual(user.first_name, 'Test')
        self.assertEqual(user.last_name, 'User')
        self.assertTrue(user.is_active)
        self.assertEqual(user.version, 1)
        
        # Check uncommitted events
        events = user.get_uncommitted_events()
        self.assertEqual(len(events), 1)
        self.assertIsInstance(events[0], UserRegistered)
    
    def test_email_change(self):
        """Test email change validation and event creation."""
        user = User(id='user-123')
        user.register('testuser', 'old@example.com')
        user.mark_events_as_committed()
        
        # Change email
        user.change_email('new@example.com')
        
        # Check state
        self.assertEqual(user.email, 'new@example.com')
        self.assertEqual(user.version, 2)
        
        # Check event
        events = user.get_uncommitted_events()
        self.assertEqual(len(events), 1)
        self.assertIsInstance(events[0], UserEmailChanged)
        self.assertEqual(events[0].old_email, 'old@example.com')
        self.assertEqual(events[0].new_email, 'new@example.com')
    
    def test_cannot_register_twice(self):
        """Test that user cannot be registered twice."""
        user = User(id='user-123')
        user.register('testuser', 'test@example.com')
        
        with self.assertRaises(ValueError):
            user.register('testuser2', 'test2@example.com')
    
    def test_cannot_change_email_when_inactive(self):
        """Test that inactive users cannot change email."""
        user = User(id='user-123')
        user.register('testuser', 'test@example.com')
        user.deactivate('Test deactivation', 'admin-456')
        user.mark_events_as_committed()
        
        with self.assertRaises(ValueError):
            user.change_email('new@example.com')

class UserProjectionTests(TransactionTestCase):
    """Tests for user projection handling."""
    
    def setUp(self):
        self.projection = UserProjectionHandler()
    
    async def test_user_registered_projection(self):
        """Test projection handles user registration."""
        event = UserRegistered(
            aggregate_id='user-123',
            aggregate_type='User',
            aggregate_version=1,
            username='testuser',
            email='test@example.com',
            first_name='Test',
            last_name='User'
        )
        
        await self.projection.handle_userregistered(event)
        
        # Check projection was created
        projection = UserProjection.objects.get(user_id='user-123')
        self.assertEqual(projection.username, 'testuser')
        self.assertEqual(projection.email, 'test@example.com')
        self.assertTrue(projection.is_active)
    
    def test_projection_sync(self):
        """Test synchronous projection handling."""
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        
        try:
            loop.run_until_complete(self.test_user_registered_projection())
        finally:
            loop.close()

@override_settings(
    EVENTUALI_CONFIG={
        'CONNECTION_STRING': 'sqlite://:memory:',
        'STREAM_CAPACITY': 100,
        'AUTO_REGISTER_EVENTS': False,
    }
)
class EventStoreIntegrationTests(TransactionTestCase):
    """Integration tests with actual event store."""
    
    async def test_full_user_lifecycle(self):
        """Test complete user lifecycle with event store."""
        # Create event store
        event_store = await EventStore.create('sqlite://:memory:')
        
        # Register events
        event_store.register_event_class('UserRegistered', UserRegistered)
        event_store.register_event_class('UserEmailChanged', UserEmailChanged)
        
        # Create and register user
        user = User(id='user-123')
        user.register('testuser', 'test@example.com', 'Test', 'User')
        
        # Save to event store
        await event_store.save(user)
        user.mark_events_as_committed()
        
        # Load user from event store
        loaded_user = await event_store.load(User, 'user-123')
        
        # Verify state
        self.assertIsNotNone(loaded_user)
        self.assertEqual(loaded_user.id, 'user-123')
        self.assertEqual(loaded_user.username, 'testuser')
        self.assertEqual(loaded_user.email, 'test@example.com')
        self.assertEqual(loaded_user.version, 1)
        
        # Change email
        loaded_user.change_email('new@example.com')
        await event_store.save(loaded_user)
        loaded_user.mark_events_as_committed()
        
        # Reload and verify
        updated_user = await event_store.load(User, 'user-123')
        self.assertEqual(updated_user.email, 'new@example.com')
        self.assertEqual(updated_user.version, 2)
        
        # Check event history
        events = await event_store.load_events('user-123')
        self.assertEqual(len(events), 2)
        self.assertIsInstance(events[0], UserRegistered)
        self.assertIsInstance(events[1], UserEmailChanged)
    
    def test_integration_sync(self):
        """Synchronous wrapper for integration test."""
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        
        try:
            loop.run_until_complete(self.test_full_user_lifecycle())
        finally:
            loop.close()

class ViewTests(TestCase):
    """Tests for Django views."""
    
    @mock.patch('users.views.get_event_store')
    async def test_user_registration_view(self, mock_get_store):
        """Test user registration view."""
        # Mock event store
        mock_store = mock.AsyncMock()
        mock_get_store.return_value = mock_store
        
        # Test POST request
        response = await self.client.post('/users/register/', {
            'username': 'testuser',
            'email': 'test@example.com',
            'first_name': 'Test',
            'last_name': 'User'
        })
        
        # Verify store.save was called
        mock_store.save.assert_called_once()
        
        # Check response
        self.assertEqual(response.status_code, 302)  # Redirect after success
```

## 🔗 Related Documentation

- **[FastAPI Integration](fastapi-integration.md)** - REST API patterns
- **[Performance Guide](../performance/README.md)** - Optimization strategies
- **[Deployment Guide](../deployment/README.md)** - Production deployment
- **[Examples](../../examples/11_django_integration.py)** - Complete Django example

---

**Next**: Try the [Pandas Integration Guide](pandas-integration.md) or explore [microservices patterns](../../examples/12_microservices_integration.py).