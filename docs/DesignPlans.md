# SolarViewer Modernization: Complete Design Plans

## Project Vision

Transform SolarViewer from a standalone Rust visualization tool into a modern, feature-complete stellar cartography platform that improves upon Astrosynthesis's capabilities. The new platform will combine:

- **Blazor WASM Frontend**: Interactive UI with 2D/3D visualization
- **Rust Backend**: High-performance "factory" for data creation, processing, and API services
- **PostgreSQL Database**: Persistent storage with PostGIS spatial capabilities
- **AI Integration**: Future-proofing for AI-assisted features
- **Advanced Calculations**: FTL travel time, spheres of influence, route optimization

---

## Architecture Overview

### High-Level System Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Web Browser (Client)                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │           Blazor WASM Application                       │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │   │
│  │  │ UI Controls  │  │ 2D/3D Maps   │  │ Calculators  │  │   │
│  │  │ (Components) │  │ (Three.js)   │  │ (FTL, etc)   │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                            ↓ HTTPS                              │
├─────────────────────────────────────────────────────────────────┤
│                     Nginx Reverse Proxy                         │
│                 (Load Balancing, SSL/TLS)                       │
└──────────┬──────────────────────────────┬──────────────────────┘
           ↓                              ↓
     ┌──────────────┐           ┌──────────────────┐
     │ Rust Backend │           │   AI Services    │
     │   (Actix)    │           │  (Future Layer)  │
     │              │           │                  │
     │  ┌────────┐  │           └──────────────────┘
     │  │ Routes │  │
     │  │ Models │  │
     │  │ Logic  │  │
     │  └────────┘  │
     │       ↓      │
     └──────────────┘
            ↓
     ┌──────────────────────┐
     │  PostgreSQL + PostGIS │
     │                       │
     │  - Star Systems       │
     │  - Routes & Waypoints │
     │  - User Data          │
     │  - Maps & Layouts     │
     └──────────────────────┘
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Frontend** | Blazor WASM | Interactive UI, real-time updates |
| **Visualization** | Three.js / WebGL | 3D star maps, 2D projections |
| **Reverse Proxy** | Nginx | HTTPS, load balancing, static files |
| **Backend API** | Rust (Actix-web) | REST/WebSocket API, calculations |
| **Database** | PostgreSQL + PostGIS | Spatial queries, data persistence |
| **Processing** | Rust (SolarViewer core) | Data import, transformations, algorithms |
| **AI Integration** | OpenAI/Claude API (future) | Procedural generation, suggestions |

---

## Design Areas (To Be Detailed)

### 1. Blazor Frontend Architecture
**Status**: Awaiting design phase

Key questions to address:
- Component hierarchy and state management
- How to handle real-time map updates from backend
- 2D vs 3D rendering strategy (when to use which)
- Navigation and zoom/pan controls
- Performance optimization for large datasets

### 2. Rust Backend (Factory Layer)
**Status**: Awaiting design phase

Key components:
- API endpoint structure (REST vs WebSocket)
- Data validation and transformation pipeline
- Integration with existing SolarViewer code
- Connection pooling and performance
- Error handling and logging

### 3. PostgreSQL Schema Design
**Status**: Awaiting design phase

Key entities:
- Maps and map collections (metadata, ownership)
- Star systems, planets, moons (imported data)
- Routes and waypoints (navigation)
- User data and preferences
- Spatial indexing strategy

### 4. FTL/Travel Time Calculator
**Status**: Awaiting design phase

Considerations:
- Support for different FTL drive types (from Astrosynthesis)
- Jump calculations vs sustained acceleration
- Waypoint-based travel planning
- Integration with route display

### 5. Spheres of Influence System
**Status**: Awaiting design phase

Considerations:
- Sphere calculation algorithm (political, economic, gravitational)
- Visualization on map (filled circles, shading, etc.)
- Filtering and layering multiple influence zones
- Recalculation triggers

### 6. Route Calculation & Optimization
**Status**: Awaiting design phase

Considerations:
- Pathfinding algorithms (A*, Dijkstra with modifications)
- Multi-stop route planning
- Distance and time cost optimization
- Integration with FTL calculator

### 7. AI Integration Layer
**Status**: Awaiting design phase

Future capabilities:
- Procedural system generation
- Trade route suggestions
- Habitability analysis
- Campaign scenario generation

### 8. Nginx Configuration on Windows 11
**Status**: Awaiting implementation

Requirements:
- SSL/TLS certificate management
- Static file serving (Blazor assets)
- Reverse proxy to Rust backend
- Optional load balancing setup

---

## Implementation Phases

### Phase 0: Foundation & Setup (Current)
- ✅ Requirements gathering and architecture planning
- ⏳ Nginx installation and configuration on Windows 11
- ⏳ Blazor WASM project setup
- ⏳ Rust backend project restructuring

### Phase 1: Core Platform (2-3 weeks estimated)
- Import existing SolarViewer functionality into new architecture
- Basic Blazor frontend with map display
- REST API for data retrieval
- PostgreSQL schema and migrations

### Phase 2: Visualization & UI (1-2 weeks estimated)
- 2D star map rendering in Blazor
- Navigation and zoom/pan controls
- Star system details panel
- Route display overlay

### Phase 3: Calculators (1-2 weeks estimated)
- FTL travel time calculator component
- Integration with route planning
- Display in UI alongside maps

### Phase 4: Advanced Features (2-3 weeks estimated)
- Spheres of influence calculation and display
- Route optimization algorithms
- User map management and persistence

### Phase 5: AI Integration (2-4 weeks estimated)
- API contracts with AI services
- Procedural generation features
- Intelligent suggestions

### Phase 6: Polish & Optimization (1-2 weeks estimated)
- Performance tuning
- UX refinement
- Documentation and deployment

---

## Design Questions to Answer

Before implementation, we need to finalize decisions on:

1. **Frontend State Management**: Redux, MobX, or Blazor's built-in approach?
2. **Real-time Updates**: WebSockets or periodic polling?
3. **3D Library**: Three.js, Babylon.js, or Cesium.js?
4. **Map Storage**: Single workspace or multiple named maps?
5. **AI Provider**: OpenAI, Claude, or self-hosted?
6. **Authentication**: Simple user system or OAuth integration?
7. **Data Sharing**: Public/private/shared map support?
8. **Export Formats**: SVG, PNG, JSON, custom formats?

---

## Windows 11 Nginx Setup Notes

### Installation Options

**Option 1: Windows Package Management (Recommended)**
```powershell
# Install via chocolatey
choco install nginx

# Or via winget
winget install nginx.nginx
```

**Option 2: Manual Download & Installation**
- Download from nginx.org official site
- Extract to `C:\nginx`
- Create Windows service wrapper

**Option 3: Windows Subsystem for Linux (WSL2)**
- Install WSL2
- Run Nginx in Linux environment
- Access via `localhost`

### Configuration Approach (To Be Detailed)
- SSL/TLS setup with self-signed certs (dev) or Let's Encrypt (prod)
- Static file serving for Blazor WASM assets
- Reverse proxy to Rust backend (e.g., `localhost:8000`)
- CORS headers configuration
- Optional rate limiting

---

## Success Criteria

### Functional
- Can load Astrosynthesis .AstroDB files and display stars
- 2D and 3D map rendering works smoothly
- FTL calculator produces reasonable results
- Routes are calculated and displayed
- Spheres of influence are visualized

### Performance
- <1s page load time
- <100ms response for typical API calls
- Smooth 60fps map panning/zooming
- Handles 1000+ star systems without lag

### User Experience
- Intuitive navigation
- Clear visual hierarchy
- Responsive design (works on various screen sizes)
- Helpful error messages

### Code Quality
- Well-documented architecture
- Comprehensive test coverage
- Clear separation of concerns
- Easy to extend and maintain

---

## Next Steps

1. **Review and finalize this design document**
   - Clarify any unclear sections
   - Answer the design questions listed above
   - Get approval before proceeding

2. **Proceed to detailed design phases**
   - Each component area will get a detailed design document
   - Include code sketches, data structures, algorithms
   - Finalize before implementation

3. **Set up development environment**
   - Nginx on Windows 11
   - Blazor WASM project
   - Rust backend project
   - PostgreSQL database

4. **Begin Phase 0 implementation**
   - Get all tools installed and configured
   - Create project repositories/structure
   - Verify all pieces can communicate

---

## Document Metadata

- **Created**: 2025-10-31
- **Status**: Draft - Awaiting Design Review
- **Version**: 1.0
- **Next Review**: After design Q&A completion
- **Related Documents**: PROJECT.md, README.md, Thoughts.md

---

*This document serves as the master blueprint for the SolarViewer modernization project. All subsequent design documents and implementation work should reference and align with these plans.*
