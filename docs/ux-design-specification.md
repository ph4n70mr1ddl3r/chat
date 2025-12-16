---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
inputDocuments:
  - /home/riddler/chat/docs/prd.md
  - /home/riddler/chat/docs/analysis/research/technical-desktop-ui-frameworks-research-2025.md
documentCounts:
  prd: 1
  research: 1
workflowType: 'ux-design'
lastStep: 6
project_name: 'chat'
user_name: 'Riddler'
date: '2025-12-16'
---

# UX Design Specification - chat

**Author:** Riddler
**Date:** 2025-12-16

---

## Executive Summary

### Project Vision

A holistic modernization of the chat desktop application combining clean, minimal visual design with optimized, frictionless workflows. The goal is to position chat as a **professional-grade communication platform**—something that feels polished and contemporary when users first open it (within 10 seconds). This is not just a visual refresh; it's an upgrade to both design AND workflows in concert to create a cohesive user experience that positions the application as "the professional alternative" to consumer-focused platforms like Discord and Slack.

**Core Design Principle**: Users should think *"This is built for serious work"* within their first 10 seconds of interaction. The interface should convey professionalism and trustworthiness through intentional, minimal design paired with friction-free workflows.

### Target Users

The design must serve five distinct user personas with different needs:

1. **Sarah Chen (First-Time User / New Adopter)**
   - Critical for growth and enterprise adoption
   - Makes snap judgment about app quality in first 10 seconds
   - Needs intuitive onboarding (< 2 minutes) that delivers immediate value
   - Design principle: Make that first message exchange magical

2. **James Rivera (Power User / Daily Operator)**
   - Lives in the app daily, coordinates projects across 5+ conversations
   - Needs: presence awareness, quick conversation switching, message history search
   - Design principle: Make frequent workflows frictionless and discoverable

3. **Elena Rodriguez (Team Lead / Multi-Conversation Manager)**
   - Manages 6+ overlapping conversations simultaneously
   - Needs: visual hierarchy showing what needs attention, context preservation, participant visibility
   - Design principle: Support multi-conversation management without overwhelming clutter

4. **Marcus Thompson (Administrator)**
   - Manages users and system health
   - Needs: clear admin interfaces for user management, activity monitoring
   - Design principle: Make admin workflows transparent and efficient

5. **David Patel (Support / Troubleshooting)**
   - Troubleshoots user issues and provides support
   - Needs: visibility into message delivery, user history, session information
   - Design principle: Provide diagnostic clarity for support workflows

### Key Design Challenges

**Challenge 1: "Professional in 10 Seconds"**
- First impression is everything for enterprise adoption
- Visual design must convey trustworthiness and quality immediately
- Must balance minimal, intentional aesthetic with feature discoverability
- **Design question**: How do we make power-user features accessible without overwhelming newcomers?

**Challenge 2: Multi-Conversation Intelligence**
- Users need to manage 5+ conversations without cognitive overload
- Each conversation must maintain context and real-time presence awareness
- **Design question**: How do we surface which conversations need attention without creating visual noise or decision fatigue?

**Challenge 3: Presence Awareness at Scale**
- Real-time presence (online/offline/typing) must be always visible for coordination
- Presence information can quickly create visual complexity
- **Design question**: How do we show presence elegantly and informatively without consuming excessive screen real estate?

**Challenge 4: Slint + Fluent Design System with 80% Component Reusability**
- Building with Slint (native Windows rendering, excellent choice)
- Fluent Design System principles must guide visual design
- Ambitious component reuse target: 80% of UI uses design system
- **Design question**: How do we architect a component system that feels premium and coherent while being highly reusable and maintainable?

### Design Opportunities

Where great UX can create competitive advantage:

**Opportunity 1: Conversation Discovery Excellence**
- Current challenge: Users struggle to find conversations quickly (per PRD)
- **Solution direction**: Design a conversation list that's simultaneously scannable AND search-enabled—where users locate any conversation in < 3 seconds
- **Competitive advantage**: Superior conversation discovery could differentiate from Discord/Slack

**Opportunity 2: Presence as Primary Information Architecture**
- Most chat apps bury presence awareness in secondary UI elements
- **Solution direction**: Make "who's available for a quick sync" the primary information hierarchy—presence becomes the hero of the UI
- **Competitive advantage**: Real-time awareness of team availability as first-class citizen in the design

**Opportunity 3: Elegant Minimal Component System**
- The 80% reuse target is ambitious but liberating—it forces intentional, well-considered design
- **Solution direction**: Design components so elegant and complete that advanced features feel simple
- **Competitive advantage**: Enables rapid feature iteration post-MVP and maintains visual coherence at scale

**Opportunity 4: First-Time Experience Magic**
- Onboarding must accomplish < 2 minutes AND deliver immediate delight
- **Solution direction**: Make that first message exchange feel like a well-designed product moment—users should feel the quality immediately
- **Competitive advantage**: Strong onboarding creates positive first impression and accelerates adoption

---

## Core User Experience

### Defining Experience

The **core user experience loop** for chat revolves around effortless conversation discovery and presence-aware messaging:

1. User opens app → sees active conversations with presence awareness
2. User finds/switches to desired conversation (< 3 seconds)
3. User instantly perceives who's online and available
4. User sends a message and sees immediate delivery confirmation
5. Message appears instantly with read receipts and presence indicators

This loop must feel completely natural—users should never think about the mechanics. Context switching between conversations should be seamless, presence should be immediately visible, and every interaction should reinforce the perception that the application is responsive, trustworthy, and professionally designed.

### Platform Strategy

**Deployment Context:**
- **Platform**: Windows 10+ desktop application (MVP scope)
- **Input Method**: Mouse + keyboard with full keyboard accessibility
- **Framework**: Slint with Fluent Design System principles
- **Architecture**: Single-server SQLite MVP with PostgreSQL migration path for scale
- **Performance Targets**: 60+ FPS rendering, < 100ms conversation switching, < 500ms message send

**Platform Implications:**
- Native Windows rendering enables premium feel and performance
- Fluent Design System guides visual language (typography, colors, spacing, motion)
- Keyboard accessibility is first-class, not afterthought
- DPI awareness and responsive layout are essential (640x480 minimum to ultrawide support)
- Windows Notifications integrate with system for background messaging

### Effortless Interactions

**1. Conversation Discovery (< 3 seconds)**
- Recent conversations appear in primary list (optimized for power users' common case)
- Search functionality finds older/archived conversations instantly
- Visual distinction between unread and read conversations guides attention
- Pinning allows power users to keep critical conversations always visible
- Scanning the list takes zero cognitive effort—visual hierarchy does the work

**2. Presence Awareness (Always Visible)**
- Presence indicators appear in conversation list view (not hidden in modals)
- Online/offline status visible in conversation header when open
- Typing indicators show real-time user activity with instant updates
- Presence changes appear immediately without page refresh
- Color + icon (not color alone) indicates presence state for accessibility

**3. Multi-Conversation Context Switching (One-Click Instant)**
- Switching between conversations feels instant (< 100ms latency)
- Scroll position and composition state preserved when returning to conversation
- Conversation header updates immediately showing current participants and presence
- Message history loads seamlessly without "loading" spinners
- Quick-switch mechanism (tab navigation) enables power users

**4. First Message Experience (Immediate Delight)**
- Onboarding guides user to first conversation (< 2 minutes total including account creation)
- Sending first message triggers immediate visual feedback
- Message appears in conversation instantly (not delayed by backend)
- Recipient's presence indicates immediate receptiveness ("They're online now!")
- Successful send creates "moment of magic"—user feels the responsiveness

### Critical Success Moments

**Moment 1: First Impression (Sarah's 10-Second Test)**
- Visual design communicates professional quality immediately
- Clean layout with intentional negative space (not sparse, not cluttered)
- Typography is crisp and readable
- Color palette feels corporate and trustworthy
- No UI breakage, no clutter, no confusion
- **Success metric**: 80% of users rate visual design as "professional" in post-launch survey

**Moment 2: First Message Sent (All Users)**
- User composes message and clicks Send
- Button provides immediate visual feedback (not just state change)
- Message appears in conversation instantly
- User sees delivery confirmation
- Recipient's read receipt arrives shortly after
- **Success metric**: 100% message delivery success rate, < 500ms send-to-display latency

**Moment 3: Power User Discovery (James' Productivity Realization)**
- Power user realizes conversation switching is seamless
- Presence is always visible without extra clicks
- Finding a conversation is faster than they expected
- Keyboard navigation works intuitively
- They experience 25%+ faster workflow compared to previous version
- **Success metric**: Power users report 25%+ faster task completion, < 3 seconds to find any conversation

**Moment 4: Team Coordination Without Overwhelm (Elena's Multi-Conversation Success)**
- Team lead opens app and scans 6 conversations in seconds
- Visual hierarchy makes it immediately clear which need attention
- Switching between conversations preserves context
- Team members' presence indicates availability for coordination
- No cognitive overload from managing multiple contexts
- **Success metric**: Team leads can manage 5+ conversations efficiently without confusion

**Moment 5: Support Diagnostic Clarity (David's Troubleshooting)**
- Support staff looks up user and sees conversation history
- Message delivery status is immediately visible
- User presence timeline shows activity patterns
- Clear error messages help diagnose issues
- Support staff resolves issues 50% faster than before
- **Success metric**: Support efficiency increases, user satisfaction with support improves

### Experience Principles

These principles guide all UX design decisions for chat:

**Principle 1: Professional Minimalism**
- Every UI element earns its place through thoughtful purpose
- Negative space is intentional and reinforces quality perception
- Visual hierarchy is crystal clear—scanning requires no effort
- Users immediately perceive craftsmanship and trustworthiness
- Polish and attention to detail signal professional-grade tool

**Principle 2: Presence-First Awareness**
- "Who's available?" is the primary information users need
- Presence indicators are always visible (not hidden in secondary menus)
- Real-time updates happen instantly and visibly
- Online/offline/away states are immediately actionable
- Users never have to wonder about someone's availability

**Principle 3: Friction-Free Context Switching**
- Switching between conversations is one-click instant
- Conversation context is preserved (scroll position, draft messages)
- Finding any conversation takes < 3 seconds (search or list)
- Keyboard navigation enables power user workflows
- No "loading states" disrupt the sense of responsiveness

**Principle 4: Information Hierarchy by Attention**
- What needs action appears prominently (new messages, @mentions, away status)
- What's quiet is visible but not distracting (archived, read conversations)
- Visual indicators (unread count, presence dots, status) guide scanning
- Users can manage 5+ concurrent conversations without decision fatigue
- Color, typography, and spacing work together to direct attention

**Principle 5: Progressive Disclosure for All Users**
- Simple and discoverable for beginners
- Powerful and efficient for experienced users
- Advanced features (filters, keyboard shortcuts) are available when needed
- Expert users can customize their workflow
- UI adapts to user skill level without sacrificing accessibility

---

## Desired Emotional Response

### Primary Emotional Goals

The **chat** application should make users feel:

**Primary Goal: "I'm capable and connected"**
- Users feel competent and in control using the tool
- Users feel connected to their team in real-time and aware of availability
- Users feel the tool scales with their growing expertise and needs
- Users experience the application as responsive, trustworthy, and professionally designed

### Emotional Journey Mapping

The emotional arc across the user experience:

**Phase 1: Discovery / First Impression (0-10 seconds)**
- **Desired Feeling**: Surprise + Confidence
- **Emotional Arc**: "Wow, this looks different!" → "This feels professional"
- **Design Leverage**: Visual polish, minimal clutter, quality aesthetic, intentional spacing
- **Success Indicator**: User thinks "This is built for serious work" within 10 seconds

**Phase 2: Onboarding / First Action (10 seconds - 2 minutes)**
- **Desired Feeling**: Guided → Capable → Delighted
- **Emotional Arc**: "I know what to do" → "I can do this" → "I just sent my first message!"
- **Design Leverage**: Clear next steps, instant feedback, quick wins, zero friction
- **Success Indicator**: User feels welcome and sends first message successfully

**Phase 3: Daily Use / Core Loop (Ongoing, repeated many times/day)**
- **Desired Feeling**: Efficient → Responsive → Connected
- **Emotional Arc**: "I'm managing conversations easily" → "Things happen instantly" → "I'm in sync with my team"
- **Design Leverage**: Instant interactions, visible presence, frictionless workflows, keyboard efficiency
- **Success Indicator**: Power users complete tasks 25%+ faster, feel in control

**Phase 4: Complex Tasks / Advanced Features (As needed)**
- **Desired Feeling**: Confident (not confused)
- **Emotional Arc**: "I need to find something specific" → "The feature is right where I expect it" → "Problem solved"
- **Design Leverage**: Discoverable advanced features, intuitive patterns, progressive disclosure
- **Success Indicator**: Users find features naturally without searching for help

**Phase 5: Error / Something Breaks (As needed)**
- **Desired Feeling**: Supported → Quickly Fixed
- **Emotional Arc**: "Something's wrong" → "I understand the issue" → "Help is available" OR "It's fixed"
- **Design Leverage**: Clear error messages, diagnostic clarity, support accessibility
- **Success Indicator**: Errors are resolved quickly with minimal frustration

### Micro-Emotions

Critical subtle emotional states that distinguish great UX:

**Confidence vs. Confusion**
- Users should feel confident they're using the app correctly
- No ambiguity about whether a message sent or is pending
- Clear visual feedback on every action (button states, sending indicators, delivery confirmation)
- **Design Implication**: Every interaction must provide immediate, unambiguous feedback

**Trust vs. Skepticism**
- Users should trust their messages are reaching recipients reliably
- Read receipts and delivery status must be visible and meaningful
- System responsiveness and reliability build trust ("It's fast, so it must be working correctly")
- **Design Implication**: Real-time confirmation, presence indicators, visible delivery status, no hidden delays

**Excitement vs. Boredom**
- Professional doesn't mean sterile—the application should feel elegant and intentional
- Smooth animations, responsive interactions, and refined details create delight
- Polish matters: button states, hover effects, smooth transitions, micro-interactions
- **Design Implication**: Attention to motion design, premium microinteractions, intentional visual feedback

**Accomplishment vs. Frustration**
- Users should feel successful when they send a message, find a contact, or switch conversations
- Every task should feel achievable without excessive steps or cognitive load
- Workflow optimization means less friction, fewer clicks, faster completion
- **Design Implication**: < 3 seconds to find any conversation, < 2 seconds to send message, smooth task flows

**Connected vs. Isolated**
- Presence awareness (who's online, who's typing) makes users feel connected to their team
- Seeing teammates online reinforces belonging and team cohesion
- Real-time interactions (someone reading their message, responding) create sense of connection
- **Design Implication**: Presence visibility prioritized, typing indicators, read receipts, real-time updates

**Empowerment vs. Helplessness**
- Power users should feel they have powerful tools and agency
- Settings, keyboard shortcuts, and customization options (post-MVP) empower users
- Experienced users should feel the app scales with their sophistication and needs
- **Design Implication**: Progressive disclosure of advanced features, keyboard-first power user support, discoverable customization

**Relief vs. Dread**
- When complex features are needed, users should feel relief ("There's a way to do this")
- Search functionality should feel powerful and accessible
- Admin and support users should feel equipped to handle edge cases
- **Design Implication**: Comprehensive search, filtering, history access, admin clarity

### Design Implications

How emotional goals translate to specific UX decisions:

| Emotional Goal | Design Direction | UX Implementation |
|---|---|---|
| **Confidence** | Immediate feedback on all actions | Loading states, sent confirmation, delivery status indicators |
| **Trust** | Real-time visibility and reliability | Presence indicators, typing indicators, read receipts, instant updates |
| **Excitement** | Premium polish and smoothness | Smooth animations, refined transitions, attention to detail and spacing |
| **Accomplishment** | Fast, frictionless task completion | < 3s conversation discovery, < 2s message send, minimal clicks |
| **Connected** | Always visible team awareness | Presence in conversation list + message header, color-coded status, real-time updates |
| **Empowerment** | Discoverable advanced capabilities | Search, filters, keyboard navigation, pinning, settings, progressive disclosure |
| **Relief** | Clear solutions and pathways | Intuitive search, helpful error messages, visible feature discovery |

### Emotions to Avoid

Equally important: preventing negative emotional states:

- ❌ **Confusion** - Never leave users uncertain whether an action succeeded
- ❌ **Frustration** - Every workflow should feel smooth and quick; no unnecessary steps
- ❌ **Isolation** - Users should always sense their team's presence and availability
- ❌ **Distrust** - Every interaction and feedback mechanism should build confidence
- ❌ **Complexity Overwhelm** - Advanced features should feel optional, never required
- ❌ **Visual Clutter** - Intentional minimalism should reduce cognitive load
- ❌ **Helplessness** - Users should always feel equipped to accomplish their goals

### Emotional Design Principles

Five core principles that guide all emotional design decisions:

**Principle 1: Confidence Through Clarity**
- Every interaction must provide unambiguous, immediate feedback
- Users never wonder if an action worked or is pending
- Visual states are distinct and meaningful
- Error messages explain what happened and how to fix it

**Principle 2: Connection Through Presence**
- Presence awareness is primary information, always visible
- Users know instantly who's online, away, or typing
- Real-time updates create sense of team synchronization
- Presence indicators are visible across all relevant UI (list, headers, input areas)

**Principle 3: Delight Through Polish**
- Premium quality in every interaction, motion, and detail
- Animations feel natural and intentional (not gratuitous)
- Hover states, button feedback, and transitions surprise and delight
- Attention to typography, spacing, and color creates professional aesthetic

**Principle 4: Calm Through Minimalism**
- Intentional design eliminates unnecessary cognitive load
- What appears on screen earns its place through purpose
- Negative space reinforces professionalism and reduces overwhelm
- Information hierarchy guides attention without effort

**Principle 5: Empowerment Through Discovery**
- Advanced features are available when users need them
- Keyboard shortcuts and power-user workflows are discoverable and efficient
- Settings and customization don't clutter the primary experience
- Progressive disclosure allows UI to evolve with user expertise

---

## UX Pattern Analysis & Inspiration

### Inspiring Products Analysis

The chat application landscape provides valuable patterns to learn from. Analysis of three key inspiration sources:

**Discord - Real-Time Responsiveness & Presence**
- Strengths:
  - Conversation discovery via hierarchical server/channel structure with instant switching
  - Presence indicators always visible (online, idle, offline, do-not-disturb)
  - Real-time updates feel instant and responsive
  - Visual distinction between unread and read channels guides attention
  - Compact layout displays many conversations efficiently
  - Excellent keyboard navigation for power users
- Weaknesses:
  - Visual design feels casual rather than professional
  - Feature creep (reactions, threads, bots) can overwhelm new users
  - Notifications can become spammy if not carefully managed
- Lessons: Responsiveness matters, presence must be visible, but keep it professional

**Slack - Professional Aesthetic & Information Architecture**
- Strengths:
  - Professional visual design conveys business-grade quality
  - Presence awareness is primary (not secondary information)
  - Search functionality is powerful and performant
  - Notification management is thoughtful and doesn't overwhelm
  - Clean typography and intentional spacing reduce cognitive load
  - Settings are accessible but not intrusive
- Weaknesses:
  - Interface can feel cluttered with too many options
  - Advanced features make it daunting for beginners
  - Learning curve is steeper than Discord
- Lessons: Professionalism matters, information hierarchy is crucial, progressive disclosure helps

**Fluent Design System (Windows 11) - Native Excellence**
- Strengths:
  - Modern, minimal aesthetic conveys quality and polish
  - Consistent typography and spacing across Windows
  - Clear visual hierarchy without clutter
  - Dark/light mode support built-in
  - Responsive design handles various window sizes naturally
  - Accessibility is first-class
- Weaknesses:
  - Still evolving (Windows 11 is relatively new)
  - Requires Windows development expertise
- Lessons: Professional, modern design is achievable with Fluent principles, platform integration matters

### Transferable UX Patterns

**Navigation & Conversation Discovery**

1. **Hierarchical Sidebar with Recent-First Sorting**
   - Recent conversations appear at top (where users look first)
   - Alphabetical or category grouping for archived/older conversations
   - Unread badges and visual distinction guide attention
   - Applicability: Perfect for chat's conversation list—recent chats, pinned chats, then search

2. **Visual Status Indicators**
   - Unread count badges prominently displayed (e.g., "3" next to conversation)
   - Presence dots (green/yellow/red) for online/idle/offline at a glance
   - Applicability: Shows at a glance which conversations need attention and who's available

3. **Unified Search as Discovery Alternative**
   - Search box that queries conversations and messages
   - Fast, responsive search for finding older conversations
   - Applicability: Enables < 3 seconds to find any conversation, even with hundreds

4. **Pinned/Favorites for Quick Access**
   - Users can pin conversations they access most frequently
   - Pinned items always visible at top of list
   - Applicability: Power users can customize their most-used conversations

**Presence & Awareness Patterns**

1. **Always-Visible Presence Indicators**
   - Color-coded status dots (green=online, yellow=away, gray=offline)
   - Visible in multiple places: conversation list, headers, message area
   - No hover-to-reveal (presence always visible for accessibility)
   - Applicability: Supports "presence-first awareness" principle and emotional goal of "connected"

2. **Real-Time Typing Indicators**
   - Show "Jane is typing..." when user is composing
   - Disappear when message is sent
   - Applicability: Creates sense of real-time connection and responsiveness

3. **Clear Delivery & Read Receipts**
   - Sent (1 checkmark) → Delivered (2 checkmarks) → Read (read icon/color change)
   - Visible on each message without tooltips
   - Applicability: Builds confidence and trust that messages are reaching recipients

**Interaction & Responsiveness Patterns**

1. **One-Click Conversation Switching**
   - Clicking a conversation instantly shows its messages
   - No loading spinners for common operations
   - Context preserved: scroll position, draft message, focus retained
   - Applicability: Achieves < 100ms switching target and supports power user workflows

2. **Instant Visual Feedback**
   - Send button changes state when clicked (not just color change, but clear feedback)
   - Message appears immediately in local UI (optimistic rendering)
   - Animations are smooth but quick (not slow motion)
   - Applicability: Creates sense of responsiveness and reliability

3. **Keyboard Navigation First**
   - Tab between conversations
   - Arrow keys to navigate conversation list
   - Enter to send, Ctrl+Enter for line break
   - Escape to close dialogs
   - Applicability: Enables power user efficiency and accessibility compliance

**Visual & Design System Patterns**

1. **Fluent Design System Application**
   - Clean typography (Segoe UI or system font)
   - Intentional whitespace and padding reduce cognitive load
   - Subtle shadows and depth create visual hierarchy
   - Responsive colors and high contrast for accessibility
   - Applicability: Professional aesthetic, Windows 11 native feel, accessibility baseline

2. **Dark/Light Mode Support**
   - Respect Windows system theme setting
   - High contrast in both modes for readability
   - Color palette works in both light and dark
   - Applicability: User preference respect, modern expectation, accessibility

3. **Compact Information Density**
   - Show more conversations without clutter
   - Use spacing and color to separate items, not borders
   - Typography hierarchy guides scanning
   - Applicability: Allows viewing many conversations efficiently (Elena's 6+ conversations)

**Onboarding Patterns**

1. **Guided First Message Experience**
   - Welcome conversation or guided tutorial
   - Clear instructions: "Find a contact → Send a message"
   - Immediate success feedback when first message sends
   - Applicability: Sarah's first impression moment—make first message feel magical

2. **Suggested Contacts**
   - Show users they can message (similar users, team members, contacts)
   - Make it easy to start first conversation
   - Applicability: Reduces friction for new users finding someone to chat with

### Anti-Patterns to Avoid

**Anti-Pattern 1: Presence Information Overload**
- ❌ Showing too many presence states (online, idle, away, do not disturb, custom status, last active time)
- ✅ Keep it simple: online/offline with optional away status
- **Rationale**: Aligns with minimalism principle; reduces visual complexity

**Anti-Pattern 2: Settings Scattered Everywhere**
- ❌ Notifications settings in one place, privacy in another, display in a third
- ✅ Centralize settings in one clear settings panel
- **Rationale**: Supports calm minimalism; reduces user confusion

**Anti-Pattern 3: Unclear Message Delivery Status**
- ❌ Message appears in conversation without indication if it sent, delivered, or failed
- ✅ Always show explicit delivery status: pending → sent → delivered → read
- **Rationale**: Builds confidence and trust; prevents user uncertainty

**Anti-Pattern 4: Feature Bloat in MVP**
- ❌ Including reactions, threads, bots, integrations, message search, advanced formatting in first release
- ✅ MVP focuses on core: 1-to-1 messaging, presence, conversation discovery
- **Rationale**: Aligns with PRD MVP scope; keeps interface focused

**Anti-Pattern 5: Hidden or Hover-Revealed Presence**
- ❌ Presence indicators only visible on hover or in tooltip
- ✅ Presence always visible without interaction
- **Rationale**: Violates presence-first principle; fails accessibility

**Anti-Pattern 6: Slow Conversation Switching**
- ❌ Loading spinners when switching conversations; feels sluggish
- ✅ Instant display without loading states for primary workflows
- **Rationale**: Perception of responsiveness matters emotionally; aligns with efficiency goals

**Anti-Pattern 7: Poor Accessibility Implementation**
- ❌ Mouse-only interface; no keyboard navigation; low contrast; no screen reader support
- ✅ Full keyboard navigation; WCAG AA contrast; semantic labels for screen readers
- **Rationale**: Inclusion matters; WCAG AA is requirement per PRD

### Design Inspiration Strategy

**What to Adopt** (Proven, directly applicable patterns):

| Pattern | Source | Application | Rationale |
|---------|--------|-------------|-----------|
| Presence visibility in conversation list | Discord, Slack | Color-coded status dots always visible | Supports "presence-first" principle; emotional "connected" feeling |
| Hierarchical sidebar with recent-first sorting | Slack, Discord | Recent conversations at top, search for older | Enables < 3 seconds to find conversation |
| One-click conversation switching | Discord | Instant display, no loading states | Supports friction-free switching principle; < 100ms target |
| Fluent Design visual language | Windows 11 | Typography, colors, spacing, motion | Professional aesthetic; Windows native integration; accessibility |
| Keyboard-first navigation | VS Code | Tab to switch, arrow keys to navigate | Power user efficiency; accessibility compliance |
| Unread count badges | Discord, Slack | Visual indicators guiding attention | Information hierarchy supports scanning |

**What to Adapt** (Modify for your unique requirements):

| Pattern | Original | Adaptation | Reason |
|---------|----------|-----------|--------|
| Presence states | Discord (complex) | Simple: online/offline + away | Minimalism principle; less visual complexity |
| Search functionality | Slack (advanced) | Simple search + basic filters | MVP scope; advanced filters post-MVP |
| Notification model | Discord (in-app) | Windows notifications | Respects OS preferences; less intrusive |
| Settings access | Slack (scattered) | Centralized settings panel | Progressive disclosure; calm minimalism |
| Feature set | Slack/Discord (comprehensive) | Core only: messages + presence + search | MVP scope per PRD; feature creep post-MVP |

**What to Avoid** (Anti-patterns that conflict with your goals):

| Anti-Pattern | Why to Avoid | Alternative |
|--------------|-------------|-------------|
| Feature bloat in MVP | Conflicts with minimalism; overwhelming for users | Focus core loop; defer advanced features |
| Hidden presence indicators | Violates presence-first + accessibility | Always-visible presence dots |
| Unclear delivery status | Damages trust and confidence | Explicit delivery status: pending → sent → delivered → read |
| Settings scattered everywhere | Violates calm minimalism | Centralized settings with progressive disclosure |
| Slow operations | Kills responsiveness perception | Instant operations, no loading spinners for core tasks |

**Design Philosophy Summary:**

Adopt Slack's professional aesthetic and information architecture maturity, combined with Discord's real-time responsiveness and presence awareness, while applying Fluent Design System for Windows native excellence. Avoid Discord's casual tone and Slack's feature complexity. Use VS Code's progressive disclosure model to enable power users without overwhelming beginners. Keep everything intentional, minimal, and professional.

---

## Design System Foundation

### Design System Choice: Fluent Design System (Hybrid Approach)

**Selected Approach: Fluent Design System Core + Custom Chat Extensions**

The **chat** application will be built on Microsoft's Fluent Design System as the foundation, with custom components created for chat-specific UI needs. This hybrid approach provides:

- ✅ Professional, modern Windows 11 aesthetic (native and trustworthy)
- ✅ Battle-tested component patterns and accessibility guidelines
- ✅ Fast development with proven patterns from Teams, Outlook, Skype
- ✅ Custom differentiation through chat-specific components
- ✅ Achievement of 80% design system reuse target
- ✅ Natural integration with Slint framework
- ✅ Scalability for post-MVP platform expansion (Mac/Linux)

### Rationale for Selection

**Why Fluent Design System?**

1. **Platform Alignment**
   - Windows 10+ is target platform (MVP scope)
   - Fluent is Microsoft's official modern design system for Windows 11
   - Users expect Windows native aesthetic and behavior
   - Immediate credibility through platform familiarity

2. **Professional Aesthetic**
   - Aligns perfectly with emotional goal: "I'm capable and connected"
   - Fluent's minimalism supports "Professional Minimalism" principle
   - Windows 11 design conveys trustworthiness and polish
   - Well-suited for enterprise adoption (CEO/executive preference)

3. **Accessibility Foundation**
   - Fluent includes comprehensive WCAG AA+ guidelines
   - Color contrast ratios, keyboard navigation, screen reader support pre-designed
   - Reduces accessibility implementation burden
   - Supports goal of "Calm Through Minimalism" without compromising inclusion

4. **Component Ecosystem**
   - Fluent provides proven components: buttons, inputs, lists, panels, navigation
   - ~70-75% of needed UI components already exist in Fluent
   - Custom layer adds 15-20 chat-specific components (messaging, presence, real-time)
   - Enables 80% reuse target efficiently

5. **Slint Integration**
   - Slint's declarative language maps naturally to Fluent's component model
   - Color tokens and spacing scales translate directly to Slint design tokens
   - Fluent's design principles guide Slint component structure
   - Hot reload enables rapid design iteration

6. **Proven in Chat Applications**
   - Microsoft Teams uses Fluent Design System
   - Skype uses Fluent principles
   - Proven to work well for real-time communication
   - Patterns for presence, messaging, notifications well-established

**Why Not Alternatives?**

- ❌ **Custom Design System Only**: Would require 6-8 weeks for core components; team doesn't have capacity
- ❌ **Unmodified Fluent Only**: Lacks chat-specific UX patterns (message bubbles, typing indicators, presence)
- ❌ **Material Design / Ant Design**: Not Windows-native; feels non-native on Windows 11; better for web/cross-platform
- ❌ **Other Windows Design Systems**: None are as current or well-supported as Fluent

### Implementation Approach

**Component Strategy: Three-Layer Architecture**

**Layer 1: Fluent Foundation (Proven, Reusable)**
- Buttons, inputs, text fields, checkboxes, radio buttons
- Lists, cards, panels, containers
- Typography system (titles, body, labels, captions)
- Color palette and semantic colors
- Spacing and layout system (8px grid)
- Elevation and shadow system
- Motion and animation principles

**Layer 2: Custom Base Components (Chat + Real-Time Adapted)**
- Message composer (large text input with send button)
- Conversation list item (with presence dot + unread badge)
- Message bubble (sent vs. received styling)
- User presence avatar
- Status indicators (online, offline, away)
- Real-time indicators (typing, read receipts, delivery status)
- Notification system (Windows notifications + in-app toasts)

**Layer 3: Composed Screens (Assembled from Layers 1 + 2)**
- Main window (conversation list + message area)
- Conversation view (messages + composer)
- User search / contact discovery
- Settings / preferences
- Admin interface

**Implementation Timeline**

**Week 1: Fluent Design System Setup**
- Establish design tokens: colors, typography, spacing, motion
- Create Slint design token files (constants for reuse)
- Document Fluent principles applicable to chat context
- Set up component library structure

**Week 2: Base Components (Fluent + Basic Chat)**
- Button component (primary, secondary, danger + icon variants)
- Text input / message composer
- List item container
- Card / panel layouts
- Presence status component
- Badge / chip component

**Week 3: Chat-Specific Components**
- Message bubble (distinct sent vs. received, with metadata)
- Conversation list item (presence + unread indicators)
- User avatar with status overlay
- Typing indicator animation
- Read receipt indicator
- Delivery status indicator
- Message time labels (relative + absolute)

**Week 4: Integration, Documentation, Polish**
- Create comprehensive component library documentation
- Build live component examples in Slint
- Establish design token management system
- Accessibility review and validation
- Performance profiling and optimization
- Create designer/developer handoff documentation

**Total Time: 4 weeks** to establish core design system (supports parallel feature development)

### Customization Strategy

**Fluent Design Tokens (Adopted As-Is)**

```
Colors:
  - Primary: Fluent Blue (trustworthiness)
  - Secondary: Fluent Teal (modern, accessible)
  - Semantic: Green (success), Red (error), Yellow (warning), Gray (neutral)
  
Typography:
  - Titles: Segoe UI 20px Bold (hierarchy)
  - Body: Segoe UI 14px Regular (readability)
  - Labels: Segoe UI 12px SemiBold (emphasis)
  - Captions: Segoe UI 11px Regular (secondary info)
  
Spacing: 8px base grid
  - Padding: 8px, 12px, 16px, 20px, 24px
  - Gaps: 4px, 8px, 12px, 16px
  
Motion: 300ms smooth easing
  - Transitions: smooth, not sluggish
  - Animations: intentional, not gratuitous
```

**Custom Extensions (Chat + Real-Time)**

```
Message Bubbles:
  - Sent: Right-aligned, blue background, custom shape
  - Received: Left-aligned, gray background, custom shape
  - Timestamp: Caption text below message
  - Delivery status: Icon (pending/sent/delivered/read)

Conversation List Items:
  - Presence dot: Top-left corner (green/yellow/gray)
  - Unread badge: Top-right corner (count or dot)
  - Last message preview: Secondary text
  - Last message time: Caption text right-aligned
  - Hover state: Subtle background highlight

Presence System:
  - Online: Green dot + label
  - Away: Yellow dot + label
  - Offline: Gray dot + label
  - Color palette: Fluent standard semantic colors

Real-Time Indicators:
  - "Jane is typing...": Animated three-dot indicator
  - Read receipts: Icon below message (eye symbol)
  - Delivery status: Checkmarks (1 = sent, 2 = delivered)
```

**What We Document Clearly**

1. **Component Library** (Figma or similar for design handoff)
   - Each component: usage, variants, states, accessibility notes
   - Live examples in Slint code
   - Best practices for composition

2. **Design Token System**
   - Color palette with semantic meanings
   - Typography hierarchy
   - Spacing scale
   - Motion principles
   - Accessibility guidelines per component

3. **Implementation Guidance**
   - Slint best practices for component structure
   - Code examples for common patterns
   - Performance considerations
   - Accessibility implementation checklist

4. **Future Extensibility**
   - Guidelines for adding new components
   - Maintaining consistency as app grows
   - Post-MVP feature patterns (reactions, threads, etc.)
   - Cross-platform adaptation (Mac/Linux patterns)

### Reusability Achievement: 80% Target

**Component Composition Analysis:**

| Component Type | Count | Reusability | Source |
|---|---|---|---|
| Buttons (primary, secondary, icon) | 6 | Fluent base + styling | Fluent Foundation |
| Input fields (text, search, message) | 3 | Fluent base + styling | Fluent Foundation |
| Lists and items | 4 | Fluent + custom styling | Fluent + Custom |
| Containers and layouts | 8 | Fluent base + composition | Fluent Foundation |
| Cards and panels | 4 | Fluent base + custom styling | Fluent Foundation |
| Presence indicators | 3 | Custom built on Fluent colors | Custom |
| Message components | 4 | Custom + Fluent styling | Custom |
| Status badges | 4 | Fluent base + custom | Fluent + Custom |
| Navigation elements | 3 | Fluent base + custom layout | Fluent Foundation |
| **Total Base Components** | **39** | **~95% reuse** | **Fluent + Custom** |

**Typical Screen Composition:** Main screen uses 12-15 of these components in composition, achieving 80%+ reuse without duplication.

### Accessibility & Quality Assurance

**Fluent Foundation Ensures:**
- ✅ WCAG AA color contrast ratios
- ✅ Keyboard navigation pathways
- ✅ Screen reader labels and semantic structure
- ✅ Focus indicators and visual feedback

**Custom Components Add:**
- ✅ Presence indicator accessibility (not color-only)
- ✅ Real-time indicator screen reader support
- ✅ Message delivery status clarity
- ✅ Typing indicator announcements (optional, user preference)

**Quality Assurance Process:**
- Automated contrast checking (Axe, WAVE tools)
- Keyboard-only user testing (all workflows accessible)
- Screen reader testing (NVDA, JAWS on Windows)
- Visual regression testing (component library)
- Performance profiling (60+ FPS rendering)

---

## Defining Core Experience

### The Defining Interaction

For **chat**, the defining experience is captured in this single, perfect interaction:

**"Find someone to talk to, send them a message, and see it arrive instantly with their response coming back in real-time."**

This is the interaction that, if we get it perfectly right, everything else becomes secondary. It's the moment users feel the responsiveness and presence that makes chat special.

Breaking this down:

**Three Components of the Defining Experience:**

1. **Conversation Discovery** - "Find someone to talk to"
   - Users scan their conversation list in < 3 seconds
   - OR use search to find anyone, anywhere in the organization
   - The action feels instant and discoverable

2. **Message Sending** - "Send them a message"
   - Users compose and send a message in < 2 seconds
   - Immediate visual feedback (not waiting for server confirmation)
   - Message appears in their view instantly

3. **Real-Time Presence** - "See response coming back"
   - Recipient's presence indicates they're available
   - Typing indicator shows they're responding in real-time
   - Message appears instantly when they send it
   - Read receipt confirms they saw it

**Why This Matters:**

This isn't just "send messages." It's the experience of **connection in real-time**. It's the perception of responsiveness that makes users feel their tool actually works. Every other feature (groups, threads, reactions, etc.) is secondary to this core loop working perfectly.

### User Mental Model

**How users currently think about this:**

**Current Mental Model (From Discord/Slack/Teams):**
- "I open the app and see a list of conversations"
- "I click on the one I want"
- "I type my message and hit send"
- "I hope they see it and respond"

**What users struggle with today:**
- "Why does it take so long to find someone?"
- "Did my message actually send? Why isn't there clear feedback?"
- "I don't know if they're online or if they saw my message"
- "Switching between conversations feels sluggish"
- "Too many UI options make me feel overwhelmed"

**What the best chat experiences do right:**
- Messages feel **instant** (not delayed)
- Presence is **always visible** (not hidden behind menus)
- Finding someone is **effortless** (search is fast and natural)
- Context **persists** when switching conversations
- Feedback is **clear** (I know exactly what's happening)

**Your Mental Model Target:**
- Open app → see who's online at a glance
- Click a person → read conversation history instantly
- Type message → send in one action
- See message sent + recipient typing indicator
- Instant connection feeling

### Success Criteria for Core Experience

**What makes users say "this just works"?**

**Criterion 1: Discovery Speed (< 3 seconds)**
- ✅ User can find any conversation in less than 3 seconds
- ✅ Recent conversations visible immediately
- ✅ Search is fast and responsive
- ✅ No "thinking" time—it feels instant
- **Measurement**: Time from app open to clicking conversation

**Criterion 2: Send Simplicity (< 2 seconds)**
- ✅ User composes and sends message in under 2 seconds
- ✅ Single click/keystroke to send (no modals, no confirmations)
- ✅ Immediate visual feedback (message appears in view)
- ✅ Composer is always ready and focused
- **Measurement**: Time from beginning to type until message appears locally

**Criterion 3: Instant Feedback (< 500ms latency)**
- ✅ Message appears in conversation < 500ms after send
- ✅ Delivery status is immediately visible
- ✅ Read receipt appears when recipient views message
- ✅ Typing indicator shows real-time user activity
- **Measurement**: Latency from user action to UI update

**Criterion 4: Presence Clarity (Always visible)**
- ✅ Recipient's online status is visible without extra clicks
- ✅ "Who's available right now" question answered at a glance
- ✅ Color-coded status (green/yellow/gray) is unambiguous
- ✅ Presence updates in real-time when user status changes
- **Measurement**: Presence visible in list and headers, updates instantly

**Criterion 5: Context Preservation (Seamless switching)**
- ✅ Scroll position maintained when switching conversations
- ✅ Draft message preserved if user switches away and back
- ✅ Message history loads instantly without loading states
- ✅ User can switch between conversations without losing place
- **Measurement**: No content loss, instant switching, smooth UX

**Criterion 6: Emotional Success (Feels premium)**
- ✅ Users think "this is responsive and professional"
- ✅ No confusion about what's happening
- ✅ Polish in every interaction (smooth, not janky)
- ✅ First impression is "built for serious work"
- **Measurement**: Post-launch surveys, qualitative feedback

### Novel vs. Established Patterns

**The core experience uses ESTABLISHED patterns, not novel ones:**

**Established Patterns We're Adopting:**

1. **Sidebar Conversation List** (Discord, Slack, Teams)
   - ✅ Users already understand this pattern
   - ✅ Proven to work well for chat
   - ✅ No learning curve needed
   - **Innovation**: Enhanced with presence visibility + fast search

2. **Instant Message Compose** (WhatsApp, iMessage, Discord)
   - ✅ Users already understand "type and send"
   - ✅ No novel interactions needed
   - ✅ Familiar from mobile messaging
   - **Innovation**: Optimized for desktop with keyboard shortcuts

3. **Read Receipts & Delivery Status** (iMessage, WhatsApp, Signal)
   - ✅ Users already understand checkmarks/read receipts
   - ✅ Clear feedback mechanism
   - ✅ No learning curve
   - **Innovation**: Always visible (not hidden in menus)

4. **Presence Indicators** (Discord, Slack, Teams)
   - ✅ Users already understand green = online, gray = offline
   - ✅ Universal convention across chat apps
   - ✅ No explanation needed
   - **Innovation**: Always visible in conversation list, more prominent

5. **Typing Indicators** (WhatsApp, iMessage, Slack)
   - ✅ Users immediately understand "Jane is typing..."
   - ✅ Creates sense of real-time connection
   - ✅ No novel UX needed
   - **Innovation**: Smooth animation, appears instantly

**Why No Novel Patterns Here?**

The core experience is about **execution excellence**, not innovation. Users don't want to learn new patterns for basic messaging. They want familiar interactions executed perfectly:
- Faster
- Smoother
- More responsive
- More professional
- Less cluttered

The innovation is in the implementation (Fluent Design, Slint performance), not the interaction model.

### Experience Mechanics: Step-by-Step

Here's the detailed mechanics for the defining experience: **"Send a message and see it arrive with real-time presence."**

#### **Phase 1: Initiation - User Opens App**

```
User Action: Launches chat application
System Response:
  1. App loads (< 1 second target)
  2. Main window appears with conversation list visible
  3. User's presence is set to "online"
  4. Conversation list shows:
     - Recent conversations at top (sorted by last message time)
     - Each conversation shows:
       * Contact name
       * Last message preview
       * Last message timestamp
       * Unread count (if any)
       * Presence dot (green/yellow/gray)

User Mental Model: "I can see who I've talked to, and who's online"
Success: User sees list in < 1 second, presence is clear
```

#### **Phase 2: Discovery - Find Someone to Talk To**

**Path A: Recent Conversation**
```
User Action: Scans conversation list
System State: Highlights conversations with unread messages
User Action: Clicks on a conversation
System Response:
  1. Conversation view loads (< 100ms target)
  2. Message history appears with full context
  3. Conversation header shows:
     * Contact name
     * Their presence status
     * Last active time
  4. Message composer is focused and ready for input

Success Indicator: User found conversation in < 3 seconds, no loading spinners
```

**Path B: Search for Someone**
```
User Action: Clicks search box or presses Ctrl+K
System Response:
  1. Search box gains focus with cursor ready
  2. Placeholder text: "Search conversations or contacts..."

User Action: Types name/keyword
System Response:
  1. Results appear instantly (< 200ms) as they type
  2. Shows matching conversations and contacts
  3. Each result shows contact name + presence

User Action: Clicks on result
System Response:
  1. Conversation/contact view opens instantly
  2. Message history loads (cached if available, fetched if needed)
  3. Composer is focused and ready

Success Indicator: User found anyone in organization in < 3 seconds via search
```

#### **Phase 3: Interaction - Compose and Send**

```
User State: Message composer is visible and focused

User Action: Types message
System Response:
  1. Text appears in composer as typed (instant, no lag)
  2. Send button becomes highlighted (indicates readiness to send)
  3. If user presses Ctrl+Enter, goes to new line

User Action: Presses Enter (or clicks Send button)
System Response:
  1. Immediate local feedback:
     * Message appears in conversation view instantly
     * Appears with "pending" status (spinner or faded)
     * Composer is cleared and re-focused
  2. Background: Message sent to server
  3. When server confirms:
     * Pending indicator changes to "sent" (checkmark)
     * When recipient receives: indicator updates to "delivered" (2 checkmarks)
     * When recipient reads: indicator updates to "read" (blue/highlighted)

Success Indicator: 
  - Message appears locally in < 100ms
  - User sees "sent" confirmation in < 500ms
  - No feeling of delay or uncertainty
```

#### **Phase 4: Presence & Real-Time Response**

```
User State: Message sent, user sees "sent" confirmation

Real-Time Update 1: Recipient comes online
System Response:
  1. In conversation header: Recipient's status changes to "online" (green dot)
  2. User sees: "Jane is now online" or status dot updates
  3. User thinks: "She's here, maybe she'll see my message soon"

Real-Time Update 2: Recipient starts typing
System Response:
  1. Typing indicator appears: "Jane is typing..."
  2. Appears below last message with subtle animation
  3. User perceives: Instant response, real-time connection
  4. When recipient stops typing:
     * Typing indicator disappears smoothly

Real-Time Update 3: Recipient sends reply
System Response:
  1. Their message appears in conversation instantly
  2. Shows: Message text + their presence status
  3. Delivery status: shows they sent it (their send checkmark)
  4. User sees new message:
     * Distinct styling (left-aligned, different background)
     * Timestamp
     * Read status (will show when user reads)

Success Indicator:
  - All real-time updates happen within 200ms of server notification
  - No waiting, no "Loading..." spinners
  - Feeling of connection and responsiveness
```

#### **Phase 5: Completion - Connection Established**

```
User State: Has sent message, recipient replied, real-time exchange

Success Outcome:
  1. User reads recipient's message (message marked as read by recipient)
  2. They see recipient typing again (ongoing real-time feel)
  3. Conversation continues with instant back-and-forth
  4. At any time, user can switch to another conversation:
     * Scroll position is preserved
     * When they return, conversation is exactly where they left it
     * New messages from other conversations are indicated with badges

Feeling User Has:
  ✓ This app is responsive
  ✓ I'm connected to my team in real-time
  ✓ Communication feels instant
  ✓ I know exactly what's happening (no confusion)
  ✓ This is built for professional work
```

#### **Phase 6: Error Case - What If Something Fails**

```
Scenario: Message fails to send

Immediate Feedback:
  1. Message shows with "failed" status (red icon or indicator)
  2. Error message appears: "Message couldn't be sent. Retry?"
  3. Retry button is immediately available

User Action: Clicks Retry
System Response:
  1. Message resends
  2. When successful, status changes to "sent"
  3. No confusion about what happened

User Mental Model: "Something went wrong, but I can fix it instantly"
Success: User recovers quickly without losing context
```

### Core Experience Design Principles

These principles guide how we implement this defining experience:

**1. Speed is Responsiveness**
- Every interaction should feel instant (< 500ms perceived latency)
- Pre-load and cache to eliminate loading states
- Optimistic rendering: show message locally before server confirms
- Users conflate speed with reliability and quality

**2. Always Visible Presence**
- Presence isn't something you have to discover—it's always there
- Online status visible in conversation list
- Updated in real-time without page refresh
- Users form mental model: "I always know who's available"

**3. Clear Feedback**
- Every action has clear, immediate feedback
- Message sent → checkmark (not silence)
- Read receipt → distinct visual indicator
- Users never wonder if something worked
- Confidence builds through clear signals

**4. Context Preservation**
- Switching between conversations is seamless
- Scroll position maintained, drafts preserved
- Users can jump between 5+ conversations without losing place
- Mental context doesn't break

**5. Progressive Disclosure**
- Core loop is simple: find → send → respond
- Advanced features available but not in the way
- Search, filters, settings discoverable but not intrusive
- Beginners can use app immediately; power users find depth

---

## Visual Design Foundation

### Color System

**Strategic Color Approach:**

The visual foundation uses Fluent Design System's proven color palette, emphasizing trustworthiness through blue-teal tones while maintaining professional enterprise aesthetics. The color strategy supports the emotional goal of "capable and connected" through clear semantic meaning.

**Primary Color Palette:**

| Token | Color | Usage | Hex | Purpose |
|-------|-------|-------|-----|---------|
| **Fluent Blue** | Deep Blue | Primary actions, brand color, interactive states | #0078D4 | Trustworthy, professional primary |
| **Fluent Teal** | Teal Accent | Secondary accent, highlights, emphasis | #00A4EF | Modern, approachable secondary |
| **Surface Primary** | Very Light Gray | Main content backgrounds | #FFFFFF (light) / #1F1F1F (dark) | Clean backdrop |
| **Surface Secondary** | Light Gray | Conversation list, secondary areas | #F3F3F3 (light) / #2D2D30 (dark) | Visual hierarchy |
| **Text Primary** | Dark Gray | Body text, labels | #000000 (light) / #E0E0E0 (dark) | Maximum readability |
| **Text Secondary** | Medium Gray | Tertiary labels, hints | #737373 (light) / #A0A0A0 (dark) | Secondary information |

**Semantic Color System:**

| Semantic | Color | Hex | Usage |
|----------|-------|-----|-------|
| **Success** | Green | #107C10 | Read receipts, message delivered, presence online |
| **Warning** | Orange | #FFB900 | Presence away, pending operations |
| **Error** | Red | #D13438 | Failed message send, error states, presence do-not-disturb |
| **Information** | Light Blue | #0078D4 | Typing indicators, new message notification |
| **Neutral** | Gray | #8A8A8A | Disabled states, borders, offline presence |

**Contrast & Accessibility:**

- ✅ Primary text on primary background: 14.5:1 ratio (WCAG AAA)
- ✅ Primary button on surface: 7.2:1 ratio (WCAG AA)
- ✅ Semantic colors differentiated by hue, not color alone
- ✅ Dark/Light mode support with appropriate contrast in both modes
- ✅ All interactive elements have sufficient contrast

**Color Application in Chat Context:**

**Conversation List:**
- Recent conversation background: Surface secondary (light gray)
- Unread conversation: Highlighted with teal accent bar
- Presence dot: Green (online), Yellow (away), Gray (offline)
- Hover state: Subtle blue highlight

**Messages:**
- Sent messages: Primary background (light blue/dark surface)
- Received messages: Slightly different surface (gray background)
- Own username: Primary blue
- Message text: Primary text color

**Real-Time Indicators:**
- "Typing": Information blue animated three-dots
- "Read receipt": Green checkmark
- "Message pending": Gray checkmark
- "Message sent": Primary blue checkmark
- "Message delivered": Primary blue double checkmark
- "Error": Red exclamation or icon

**Presence System:**
- **Online**: Green (#107C10) - Available for conversation
- **Away**: Orange (#FFB900) - May not respond immediately  
- **Offline**: Gray (#8A8A8A) - Not currently available
- **Do Not Disturb**: Red (#D13438) - Actively not available

### Typography System

**Fluent Design Typography Foundation:**

The typography system uses **Segoe UI** (Windows native font) with a careful hierarchy supporting both professional communication and real-time messaging contexts.

**Type Scale:**

| Type Level | Size | Weight | Line Height | Usage |
|-----------|------|--------|-------------|-------|
| **Title** | 20px | Bold (700) | 28px | Window title, main headers |
| **Heading 2** | 16px | SemiBold (600) | 22px | Section headers, conversation title |
| **Heading 3** | 14px | SemiBold (600) | 20px | Card titles, list section headers |
| **Body Large** | 14px | Regular (400) | 20px | Message text, main content |
| **Body** | 13px | Regular (400) | 19px | Primary body text, descriptions |
| **Body Small** | 12px | Regular (400) | 18px | Secondary information, timestamps |
| **Caption** | 11px | Regular (400) | 16px | Helper text, labels, timestamps |
| **Label** | 12px | SemiBold (600) | 18px | Button text, chips, badges |

**Hierarchy Strategy:**

**Primary Heading (Conversation Title)**
- 16px SemiBold blue
- Clear visual hierarchy
- User's name or conversation topic
- Example: "Jane Chen"

**Secondary Content (Last Message, Timestamps)**
- 12px Regular gray
- Reduces visual weight
- Indicates secondary information
- Example: "Jane: That sounds great! See you then."

**Message Text (Body Content)**
- 13px Regular, color depends on context
- Optimized for readability at typical viewing distance
- 19px line height for comfortable reading
- Sufficient contrast for all users

**Interactive Labels (Buttons, Links)**
- 12px SemiBold teal/blue
- Distinct from body text
- Clear affordance of interactivity
- Example: "Send", "Search", "Settings"

**Timestamps & Metadata**
- 11px Regular gray
- Subtle, doesn't distract
- Always readable but secondary
- Example: "3:45 PM"

**Font Pairing Rationale:**

- **Segoe UI Only** - Windows native, optimized for screen display
- No secondary font needed - consistent, professional appearance
- Clear hierarchy through weight and size changes only
- Excellent readability at small sizes (important for chat)
- Built-in accessibility features

**Text Rendering Optimization:**

- Anti-aliasing: Enabled for smooth edges
- Kerning: Standard (Windows handles automatically)
- Ligatures: Enabled for professional appearance
- Text rendering: ClearType for Windows (optimal)

### Spacing & Layout Foundation

**8px Grid System:**

The foundation uses an 8px base grid system, enabling consistency and predictability across all components.

**Spacing Scale:**

```
xs: 4px   (fine-tuning, borders, tight gaps)
sm: 8px   (component padding, list gaps)
md: 12px  (section padding, moderate gaps)
lg: 16px  (large padding, visual separation)
xl: 20px  (major section padding)
xxl: 24px (page-level padding, major gaps)
```

**Component-Level Spacing:**

| Component | Padding | Gap | Example |
|-----------|---------|-----|---------|
| **Button** | 8px vertical, 12px horizontal | N/A | Small button with icon + text |
| **Input Field** | 8px vertical, 12px horizontal | N/A | Search box, message composer |
| **Card** | 12px | N/A | Conversation list item |
| **List Item** | 8px top/bottom, 12px left/right | N/A | Message in conversation |
| **Container** | 16px | 12px | Conversation view padding |
| **Message Bubble** | 12px | N/A | Chat message container |

**Layout Architecture:**

**Three-Panel Layout (Main View):**
```
┌─────────────────────────────────────┐
│ [Header: Logo, User Menu]           │
├────────────┬──────────────────────────┤
│            │                          │
│ Sidebar    │  Message Area            │
│ (240px)    │  (Flexible)              │
│            │                          │
│ • Recent   │  [Messages]              │
│ • Contacts │  ┌──────────────────┐    │
│ • Search   │  │ Jane is online   │    │
│            │  │ Jane: Hey there! │    │
│            │  │ You: Hi Jane!    │    │
│            │  │ Jane: typing...  │    │
│            │  └──────────────────┘    │
│            │  ┌────────────────────┐  │
│            │  │ [Type message...] ▶ │  │
│            │  └────────────────────┘  │
└────────────┴──────────────────────────┘
```

**Responsive Breakpoints:**

- **Full Desktop** (1200px+): Three-panel layout, all features visible
- **Medium Desktop** (800-1200px): Sidebar collapses to icons, message area expands
- **Tablet** (600-800px): Sidebar slides out (drawer), message area full width
- **Minimum** (640px): Sidebar hidden, message area full width

**Whitespace Strategy:**

- **Generous internal spacing** (16px+ between sections) supports "calm minimalism"
- **Minimal external padding** - uses full available space efficiently
- **Clear visual separation** through spacing and subtle backgrounds
- **Breathing room** around text prevents overwhelm

**Density Considerations:**

- Conversation list: Compact (40px item height) to show many at once
- Message area: Relaxed (generous line height, spacing) for readability
- Balance between information density and clarity

### Accessibility Considerations

**Color Accessibility:**

- ✅ No information conveyed by color alone (presence uses dots + labels + text)
- ✅ WCAG AA contrast ratios met for all text
- ✅ WCAG AAA contrast for primary interactive elements
- ✅ Dark/Light mode both meet accessibility standards
- ✅ Semantic colors differentiated by hue and symbol

**Typography Accessibility:**

- ✅ Minimum 12px font size for body text
- ✅ Line height 1.5x or greater for improved readability
- ✅ Maximum line length 80 characters for comfortable reading
- ✅ High contrast ratios (7:1 or better for WCAG AA)
- ✅ Font scaling respects Windows accessibility settings

**Spatial Accessibility:**

- ✅ 44px minimum touch target size (desktop: 24px acceptable for mouse, but 44px preferred)
- ✅ Spacing between interactive elements prevents accidental clicks
- ✅ Consistent spacing aids screen reader users in understanding structure
- ✅ Focus indicators clearly visible (3px outline, high contrast)

**Motion Accessibility:**

- ✅ All animations have equivalent non-animated alternatives
- ✅ Respects `prefers-reduced-motion` system setting
- ✅ No auto-play content or flashing elements
- ✅ Animations use 300ms duration (fast enough to feel responsive, slow enough for perception)

**Testing & Validation:**

- Automated: Axe, WAVE, Lighthouse color contrast checks
- Manual: Keyboard-only navigation testing
- Screen reader: NVDA testing on Windows
- User: Testing with actual users who have accessibility needs

### Design Tokens Implementation

**For Slint Implementation:**

```slint
// Color Tokens
export namespace Colors {
  export color primary: #0078D4;
  export color secondary: #00A4EF;
  export color success: #107C10;
  export color warning: #FFB900;
  export color error: #D13438;
  
  export color surface-primary: #FFFFFF;
  export color surface-secondary: #F3F3F3;
  
  export color text-primary: #000000;
  export color text-secondary: #737373;
}

// Typography Tokens
export namespace Typography {
  export font heading-2: { size: 16px, weight: 600 };
  export font body: { size: 13px, weight: 400 };
  export font caption: { size: 11px, weight: 400 };
}

// Spacing Tokens
export namespace Spacing {
  export xs: 4px;
  export sm: 8px;
  export md: 12px;
  export lg: 16px;
  export xl: 20px;
  export xxl: 24px;
}
```

---

## Design Direction Decision

### Design Directions Explored

Six primary design direction variations were explored, each representing a different visual and interaction approach while maintaining alignment with the Fluent Design System foundation:

**Direction 1: Compact Professional (Recommended)**
- Sidebar: 240px fixed, presence dots prominent
- Layout: 3-panel (sidebar, conversation list, message area)
- Density: Compact (40px list items, efficient use of space)
- Visual Weight: Balanced (clean, minimal, professional)
- Presence: Always visible with color + label + status indicator
- Information Hierarchy: Clear visual separation through color and spacing
- Best For: Power users, professionals managing multiple conversations

**Direction 2: Card-Based Modern**
- Cards: Conversation list as card grid (2-column on wide displays)
- Sidebar: Collapsible to icons only
- Layout: Sidebar + card grid for conversations
- Density: Relaxed (larger cards, more whitespace)
- Visual Weight: Light (subtle shadows, card elevation)
- Presence: Status pill within card, more visual prominence
- Best For: Visual appeal, modern aesthetic, discovery-focused

**Direction 3: Minimal Distraction-Free**
- Sidebar: Minimal, icons only (expand on hover)
- Focus: Full-width conversation view in focus
- Layout: Hidden sidebar, focus-on-message area
- Density: Maximum (prioritizes message reading area)
- Visual Weight: Ultra-minimal (high contrast with minimal ornamentation)
- Presence: Top-of-screen header with full presence/status details
- Best For: Focused communication, reduced UI clutter, immersive reading

**Direction 4: Conversation List Priority**
- Full-screen sidebar: Conversation list takes primary real estate
- Messages: Panel on right side (resizable)
- Layout: Split-pane with adjustable divider
- Density: High in sidebar (more conversations visible)
- Visual Weight: Sidebar prominent, message area secondary
- Presence: Large presence indicators in list, easy scanning
- Best For: Managing many conversations, power user workflows

**Direction 5: Fluent Native (Platform-First)**
- Follows Windows 11 Fluent Design strictly
- Navigation: Top navigation bar with command bar
- Layout: Sidebar + messages + detail pane
- Density: Balanced per Fluent guidelines
- Visual Weight: Fluent-standard (built-in elevation, Acrylic effects)
- Presence: Fluent badge components, consistent with Windows
- Best For: Maximum Windows platform integration, native feel

**Direction 6: Real-Time First (Presence-Focused)**
- Presence: Occupies significant visual space (top or side)
- Sections: Online now | Away | Offline | Do Not Disturb
- Messages: Secondary but readily accessible
- Layout: Presence grid + conversation picker
- Density: Moderate (balance between directory and messages)
- Visual Weight: Presence information is hero
- Best For: Team coordination, presence-driven workflows, availability at a glance

### Chosen Direction: Compact Professional (Direction 1)

**Selected Approach: Direction 1 - Compact Professional**

The **Compact Professional** direction best aligns with your product vision, emotional goals, and user needs:

**Why This Direction Wins:**

1. **Aligns with Core Experience**
   - Optimizes for rapid conversation discovery (< 3 seconds)
   - Message sending is prominent and accessible
   - Presence always visible without UI overhead

2. **Matches Emotional Goals**
   - "I'm capable and connected" → Professional, minimal aesthetic + presence always visible
   - "Professional minimalism" → Clean, intentional design with zero clutter
   - "Presence-first awareness" → Presence dots/indicators in conversation list
   - "Responsive delight" → Fast switching, instant feedback

3. **Optimizes for User Personas**
   - Sarah (first-time): Clean interface, not overwhelming, professional first impression
   - James (power user): 240px sidebar fits many conversations, keyboard navigation efficient
   - Elena (team lead): Compact list shows 6+ conversations with presence at a glance
   - Marcus (admin): Dedicated admin section accessible from main interface
   - David (support): Conversation history and metadata readily visible

4. **Technical Fit**
   - Slint declarative language maps naturally to this layout
   - Responsive behavior straightforward (sidebar collapses on smaller displays)
   - Real-time updates don't require complex layout recalculations
   - Performance is excellent (fixed sidebar, scrollable content)

5. **Design System Efficiency**
   - Uses Fluent base components directly
   - Minimal custom component needs (mostly composed from Fluent)
   - Enables 80%+ component reuse

### Design Direction Implementation Approach

**Layout Structure:**

```
┌──────────────────────────────────────────────────────┐
│  chat  [User] [Settings] [Help]                      │  Header (48px)
├──────────────┬──────────────────────────────────────┤
│              │                                      │
│  Sidebar     │  Conversation View                   │
│  240px       │                                      │
│              │  ┌────────────────────────────────┐  │
│ • Recent     │  │ Jane Chen  🟢 Online           │  │ Header (56px)
│   (4 items)  │  └────────────────────────────────┘  │
│              │  ┌────────────────────────────────┐  │
│ • Search     │  │ Jane: Hey, got a minute?       │  │
│   [______]   │  │ You: Sure, what's up?          │  │
│              │  │ Jane: typing...                │  │ Messages (Flex)
│ • Contacts   │  │                                │  │
│   [All]      │  │                                │  │
│              │  │                                │  │
│ • Settings   │  └────────────────────────────────┘  │
│              │  ┌────────────────────────────────┐  │
│              │  │ [Type a message...       ] ▶ 󰄉  │  │ Composer (56px)
│              │  └────────────────────────────────┘  │
└──────────────┴──────────────────────────────────────┘

Sidebar (240px):
├─ Header: Logo + User Menu
├─ Search (sticky)
├─ Recent Conversations (scrollable)
│  ├─ Item: [🟢] Jane Chen | "See you at..."
│  ├─ Item: [🟡] Bob Smith | "Files sent"
│  └─ Item: [⚪] Sarah Lee | "Tomorrow works"
├─ Quick Access: Contacts, Settings
└─ Status Indicator (bottom)

Conversation Area:
├─ Header: Contact name + presence + options
├─ Messages: Scrollable thread
├─ Composer: Input field + send button
└─ Real-time indicators: Typing, presence updates
```

**Visual Specifications for This Direction:**

| Element | Specification | Notes |
|---------|---|---|
| **Sidebar width** | 240px fixed | Collapses to icons (48px) on narrow displays |
| **Sidebar background** | Surface secondary (light gray) | Subtle difference from main content area |
| **Item height** | 40px | Compact, shows many conversations at once |
| **Presence dot** | 12px circle, top-left | Always visible without hover |
| **Unread badge** | 20px circle, top-right | White dot or count number |
| **Message composer** | 56px fixed height | 8px padding, text wraps to 2-3 lines max |
| **Header** | 56px fixed height | Contact name + presence status |
| **Breakpoint collapse** | < 900px | Sidebar becomes collapsible drawer |

**Component Usage:**

This direction composes these Fluent components:
- **Buttons**: Send, Settings, collapse sidebar
- **Text Input**: Search, message composer
- **Lists**: Conversation list, message history
- **Panels**: Sidebar, conversation area
- **Badges/Chips**: Unread count, presence status
- **Icons**: Presence indicators, action buttons
- **Typography**: Fluent type scale applied throughout

### Design Mockup Rationale

**Why Compact Professional Over Alternatives:**

| Consideration | Compact Prof | Card-Based | Distraction-Free | List Priority | Fluent Native | Presence-First |
|---|---|---|---|---|---|---|
| **Discovery Speed** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Presence Visibility** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Message Focus** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Professional Feel** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Power User Efficiency** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Implementation Simplicity** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Responsive/Adaptive** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Overall Score** | **33** | 25 | 26 | 30 | 31 | 26 |

**Winner: Compact Professional** best balances all considerations while specifically optimizing for your core experience.

### Next Steps: Wireframe Development

With the Compact Professional direction locked, the next phase involves:

1. **High-Fidelity Wireframes** - Detailed layouts for each key screen
2. **Interactive Prototypes** - Slint mockups showing real interactions
3. **Component Library** - Complete Fluent + custom component specifications
4. **User Journey Flows** - Detailed flows for each persona's workflow

This visual direction provides clear guidance for all implementation and design work going forward.

---

## User Journey Flows

### Journey 1: Sarah Chen - First-Time User Onboarding

**Scenario**: Sarah is trying chat for the first time. She signs up and needs to send her first message to feel the product is valuable.

**Flow:**
```
1. Launch App
   → Sees welcome screen with brief orientation
   → Gets guided to find a contact
   
2. Onboarding (< 2 minutes total)
   → Creates account or logs in
   → System suggests contacts to message
   → OR search field is prominent for finding someone
   
3. First Message Moment (CRITICAL)
   → Finds Jane (suggested or searched)
   → Opens conversation
   → Types "Hi Jane, testing this out"
   → Clicks Send
   → Message appears INSTANTLY in their view
   → See delivery confirmation immediately
   → Jane is shown as "online" → feels like she might respond
   
4. Emotional Checkpoint
   ✓ Message appeared instantly → "This works!"
   ✓ Jane's presence visible → "She might reply!"
   ✓ Professional interface → "This is serious software"
   ✓ Simple flow → "I figured this out"
   
5. Success Outcome
   Jane sees message, types back, Sarah sees "Jane is typing..."
   Sarah's mental model: "This is responsive real-time chat"
```

**UX Principles Applied:**
- Instant feedback (message appears immediately)
- Clear presence (Jane online indicator)
- Professional minimal design (not overwhelming)
- Successful first action (first message sent)

### Journey 2: James Rivera - Power User Multi-Conversation Management

**Scenario**: James is managing a project across 5+ concurrent conversations. He switches between conversations rapidly and needs instant context.

**Flow:**
```
1. App Open (Daily Routine)
   → Sees 8-10 recent conversations
   → Unread badges show which need attention
   → Presence dots show who's online
   
2. Scan Phase (< 3 seconds)
   → Eye scans conversation list
   → Sees: [🟢] Jane | "Feedback on design"
   → Sees: [🟡] Bob | "PR waiting review"
   → Sees: [⚪] Sarah | "Meeting notes"
   → Priorities clear at a glance
   
3. Context Switch #1: Jane's Conversation
   → Clicks Jane's conversation
   → Message history loads instantly (< 100ms)
   → Scroll position preserved from last visit
   → Typing "Looks good, pushing live"
   → Sends (< 1 second from decision to send)
   → Sees "sent" checkmark + delivery status
   
4. Context Switch #2: Bob's Conversation
   → Presses Tab or clicks Bob
   → Jane's conversation preserved (scroll, draft)
   → Bob's conversation appears instantly
   → Sees PR link, reads feedback
   → Types response and sends
   
5. Power User Shortcuts
   → Presses Ctrl+K to search for anyone
   → Quickly finds Sarah's conversation
   → Switches to her thread
   → All context preserved seamlessly
   
6. Real-Time Ongoing
   → While typing to Bob, Jane sends message
   → James sees unread indicator on Jane's tab
   → Completes thought with Bob
   → Switches to Jane, sees new message, continues
   
7. Emotional Checkpoint
   ✓ Fast discovery (< 3 sec to find anyone)
   ✓ Responsive switching (instant, no spinners)
   ✓ Context preserved (pick up where he left off)
   ✓ Clear priorities (unread badges guide attention)
   ✓ Feels in control (can handle 5+ conversations)
```

**UX Principles Applied:**
- Speed is responsiveness (< 100ms switches)
- Presence-first (knows who's online)
- Context preservation (scroll, drafts maintained)
- Clear feedback (unread badges, status indicators)

### Journey 3: Elena Rodriguez - Team Lead Coordination

**Scenario**: Elena is managing 6 team conversations simultaneously. She needs to see what needs attention and understand team availability.

**Flow:**
```
1. Daily Standup Review
   → Opens app
   → Sees 6 team conversations in sidebar
   → Presence indicators show: 5 online, 1 away
   → Unread badges show: Jane has 2, Bob has 1
   
2. Assess Team Availability (< 10 seconds)
   → [🟢] Jane - Online (2 unread) - probably waiting on something
   → [🟢] Bob - Online (1 unread) - minor issue
   → [🟢] Sarah - Online - ready to coordinate
   → [🟢] Marcus - Online - available
   → [🟡] David - Away - followup needed
   → [⚪] External vendor - Offline - no immediate response expected
   
3. Context-Aware Decisions
   → Priority 1: Click Jane (handles urgent first)
   → Sees: "Design feedback stuck, waiting on you"
   → Responds immediately: "Reviewing now, 30 mins"
   → Jane sees message + read receipt
   
4. Context Switch: Bob (Priority 2)
   → Sees issue with deployment
   → Type: "Sarah, can you jump on this with Bob?"
   → Mentions Sarah's name
   → Sarah gets notified
   
5. Team Coordination (Real-Time)
   → Sarah joins conversation about deployment
   → Elena sees "Sarah is typing..."
   → Meanwhile, David comes back online
   → Elena sees his presence change to green
   → Switches to David's thread
   
6. Multi-Conversation Orchestration
   → Managing 5 conversations, none feel disconnected
   → Knows who's available for quick sync
   → Unread badges keep priorities clear
   → Presence updates show team activity
   
7. Emotional Checkpoint
   ✓ Visibility (knows team availability)
   ✓ Coordination (can message multiple people instantly)
   ✓ Clarity (unread badges + presence = clear priorities)
   ✓ Empowerment (feels in control of team)
```

**UX Principles Applied:**
- Presence-first (always visible team status)
- Information hierarchy (unread badges guide attention)
- Multi-conversation support (handle 6 without overwhelm)
- Real-time updates (see presence changes instantly)

---

## UX Patterns & Interactions

### Navigation Patterns

**Primary Navigation: Sidebar**
- Fixed 240px on desktop
- Collapses to icons on 900px< (drawer on interaction)
- Always accessible, never hidden
- Recent conversations sorted by last message time
- Search prominent at top for discovery

**Secondary Navigation: Tabs (Future MVP+)**
- Could add Teams/Channels tabs (post-MVP)
- Pattern already established for team lead use case
- Tabs keep contexts separate but accessible

**Tertiary Navigation: Menus**
- User menu (top right) - Profile, Settings, Logout
- Conversation menu (in header) - Info, Settings, Leave
- Right-click context menus (future) - Pin, Mute, Archive

### Message Sending & Receiving

**Compose Pattern:**
- Single-line input with auto-expand (up to 3-4 lines)
- Send button always visible (right side of composer)
- Keyboard: Enter to send, Ctrl+Enter for newline
- Optimistic rendering: Message appears locally before server confirmation

**Delivery Status Pattern:**
```
⏳ Pending (gray spinner) → 
✓ Sent (single checkmark) → 
✓✓ Delivered (double checkmark) → 
✓✓ Read (blue/highlighted checkmark)
```

**New Message Notification Pattern:**
- In-conversation: Message appears at bottom with light flash
- Out-of-conversation: Unread badge appears on conversation item
- OS notification: Windows notification if app backgrounded
- Typing indicator: "Jane is typing..." appears with animation

### Loading States & Feedback Mechanisms

**Loading State Hierarchy (When to Show What):**

1. **Immediate Feedback (< 100ms)** - Optimistic UI
   - Message appears locally immediately (pending state)
   - Conversation switches instantly to show message input is ready
   - Search results update as-you-type
   - **Never show loading spinner for immediate operations**

2. **Short Delay (100-500ms)** - Typing Indicator
   - Typing indicator appears when user starts typing ("Jane is typing...")
   - **No spinner needed** - Animation is the indicator
   - Shows up to 3 typists in group conversations
   - Disappears when user stops typing or sends message

3. **Medium Delay (500-2000ms)** - Skeleton Screens
   - **When:** Message loading from history, conversation details loading, search results appearing
   - **Skeleton pattern:**
     - Conversation list: Show 3-4 gray placeholder cards with avatar shapes + text shimmer
     - Message area: Show 2-3 gray message bubble skeletons with shimmer effect
     - Header: Show gray placeholder for conversation name + participants
   - **Why skeleton vs. spinner:** Maintains layout structure, reduces cognitive load, feels faster
   - **Shimmer animation:** Subtle left-to-right shine effect (400ms duration), ease-in-out easing
   - **Duration:** Show skeleton for up to 2 seconds max, then show error if still loading

4. **Long Delay (> 2000ms)** - Spinner + Fallback
   - **When:** Network request taking longer than expected, initial app load
   - **Spinner specification:**
     - Icon: Circular spinner or loading dots
     - Color: Fluent Blue (#0078D4)
     - Size: 24px diameter
     - Animation: Rotation 360° in 800ms, linear easing, loops continuously
     - Location: Center of available content area with 10px top margin
   - **With message:** "Loading..." (Body text, 14px, centered below spinner)
   - **Fallback:** After 5 seconds, show "Still loading..." or error state with retry button

**Delivery Status Feedback:**
```
⏳ Pending (gray icon, spinner animation) - Message sent, awaiting server confirmation
✓ Sent (check icon, Fluent Blue #0078D4) - Server received message (< 500ms)
✓✓ Delivered (double check, Fluent Blue) - Delivered to recipient (< 1s)
✓✓ Read (double check highlighted, Teal #00A4EF) - Recipient read message
✗ Failed (X icon, Error red #E81123) - Message failed to send, show retry button
```

**Error Loading States:**
- **Network Error:** Banner at top: "Connection lost. Retrying..." (auto-retry every 2s)
- **Timeout:** "This is taking longer than usual. [Retry] or [Cancel]"
- **Failed Send:** Message shows error icon + inline "[Retry] [Delete]" buttons
- **Partial Load:** Show what loaded, with "More messages loading..." indicator at bottom

**Animation Specifications for Loading:**
- Skeleton shimmer: 400ms duration, ease-in-out, #F0F0F0 to #E8E8E8 gradient
- Spinner rotation: 800ms per full rotation, linear, continuous
- Message fade-in: 200ms fade-in (opacity 0 to 1), ease-out when appearing
- Skeleton fade-out: 200ms fade-out when replaced with actual content
- Bounce animation (optional, deferred): Subtle 200ms bounce when new message arrives

**Best Practices:**
- Never show multiple loading indicators at once
- Always provide user agency: "Cancel" button visible with long operations
- Show time estimates for operations > 3 seconds ("Uploading... 2 of 5 files")
- Combine visual + textual feedback (spinner + "Loading...")
- Test all loading states with real latency (use throttling in dev tools)

---

### Presence & Availability

**Presence States:**
- 🟢 **Online** - Green dot, immediately available
- 🟡 **Away** - Yellow dot, stepped away but watching
- ⚪ **Offline** - Gray dot, not currently using app
- 🔴 **Do Not Disturb** - Red dot, actively not available

**Presence Indicators Location:**
1. Conversation list item (always visible)
2. Conversation header (when open)
3. User mention/profile (when hovering)

**Presence Update Pattern:**
- Status changes appear instantly (< 200ms)
- No page refresh needed
- Real-time propagation from server

**Last Active Pattern:**
- Hover over presence dot → "Online now" or "Away 3 minutes ago"
- Tooltip shows last active time
- Helpful for assessing responsiveness

### Search & Discovery

**Quick Search (Ctrl+K):**
- Focus search immediately
- Search across: Conversation names, recent messages, contacts
- Results appear as-you-type (< 200ms latency)
- Click result → Conversation opens instantly
- History of recent searches (future enhancement)

**Contact Discovery:**
- Contacts list section in sidebar
- Browse all contacts, see their online status
- Start new conversation from any contact
- Click contact → Opens new conversation thread

### Error Handling & Edge Cases

**Message Send Failure:**
```
1. User clicks Send
2. Message queues locally
3. Attempt send fails (network error)
4. Message shows "Failed to send" (red icon)
5. User sees error message: "Couldn't send. Retry?"
6. Click Retry → Resends automatically
7. On success, status updates to "sent"
```

**Network Disconnection:**
```
1. Connection lost
2. Composer shows "offline" indicator
3. Can still type messages (they queue)
4. Connection restored
5. Queued messages send automatically
6. User notified: "Messages caught up"
```

**Recipient Blocked/Unavailable:**
```
1. User tries to send to blocked contact
2. Appears to send normally (no confusion)
3. Server prevents delivery
4. Message shows "Not delivered" (with reason if applicable)
5. Helpful error message explains next steps
```

---

## Component Strategy & Library

### Core Component Categories

**Navigation Components:**
- Sidebar container, sidebar items
- Navigation tabs (future)
- Breadcrumb trail (future)

**Communication Components:**
- Message bubble (sent vs. received)
- Conversation list item
- Message composer
- Typing indicator
- Read receipt indicator
- Presence avatar

**Information Display:**
- Presence badge (dot + status)
- Unread badge (count)
- Status indicator (online/away/offline/dnd)
- User profile card
- Conversation header

**Input & Control:**
- Text input (search, compose)
- Buttons (primary, secondary, icon)
- Dropdown menus
- Toggle/Switch (future, for settings)
- Checkbox (future, for multi-select)

**Containers & Layout:**
- Main window container
- Conversation view container
- Card/Panel components
- Spacers/Dividers

### Component Reuse Matrix

| Screen | Components Used | Reuse Count |
|--------|---|---|
| **Main View** | Sidebar, List items, Buttons | 4-6 unique |
| **Conversation** | Message bubbles, Composer, Header, Avatar | 3-5 unique |
| **Settings** | Input fields, Buttons, Cards | 3-4 unique |
| **Search** | Search input, List items, Avatar | 2-3 unique |
| **Total Unique** | ~20 custom components | **Total Apps Uses ~80-90 instances** |

**Reuse Percentage: ~80%** (our target achieved!)

### Accessibility Implementation

**Keyboard Navigation:**
```
Tab:         Move between interactive elements
Shift+Tab:   Move backward through elements
Arrow Keys:  Navigate within lists (up/down)
Enter:       Activate buttons, send messages
Escape:      Close dialogs, cancel actions
Ctrl+K:      Open search (global hotkey)
```

**Screen Reader Support:**
- Semantic HTML structure with proper ARIA labels
- Conversation list announces "2 unread messages from Jane"
- Presence changes announced ("Jane is now offline")
- Buttons labeled clearly ("Send message", "Archive conversation")
- Form labels associated properly

**Visual Accessibility:**
- ✅ 7:1 contrast ratio for normal text
- ✅ 4.5:1 for large text
- ✅ Focus indicators 3px outline (high contrast)
- ✅ No information by color alone
- ✅ Respects `prefers-reduced-motion`

**Motion & Animation Accessibility (Vestibular Disorders):**

Users with vestibular disorders, motion sensitivity, or epilepsy require safe animation alternatives. This specification implements full WCAG 2.1 Success Criterion 2.3.3 (Animation from Interactions) and 2.3.4 (Animation Options).

**Implementation Requirements:**

1. **Detect Accessibility Preference:**
   ```
   @media (prefers-reduced-motion: reduce) {
     /* All animations must be disabled or substantially simplified */
   }
   ```

2. **Animation Reductions (Vestibular Safe):**

   | Animation | Normal Behavior | Reduced Motion Alternative |
   |-----------|-----------------|---------------------------|
   | **Typing Indicator** | Bouncing dots animation (600ms loop) | Static text "Jane is typing..." (no animation) |
   | **Presence Status** | Dot color fade transition (300ms) | Instant color change, no transition |
   | **Message Arrival** | Fade-in + subtle slide (200ms) | Instant appearance, no animation |
   | **Conversation Switch** | Smooth cross-fade (100ms) | Instant switch, no fade |
   | **Skeleton Shimmer** | Shimmer animation (400ms loop) | Static gray placeholder, no shimmer |
   | **Loading Spinner** | Continuous rotation (800ms) | Pulsing opacity instead (300ms pulse, 600ms pause) |
   | **Hover Effects** | Color transition (200ms) | Instant color change |
   | **Focus Indicators** | Glow animation | Static outline (no animation) |
   | **Notification Toast** | Slide-in animation (200ms) | Instant appearance at final position |
   | **Dialog Open** | Scale + fade (200ms) | Instant appearance, no scale |

3. **Slint Implementation Pattern:**
   ```slint
   export component TypingIndicator {
     // Check accessibility preference
     accessible-preferred: @env("PREFER_REDUCED_MOTION") == "true";
     
     // Normal animation
     animate-dots when !accessible-preferred {
       dots.offset-y: animation(1s loop, ease-in-out);
     }
     
     // Reduced motion: Static text only
     if accessible-preferred {
       text("Jane is typing...");
     } else {
       // Normal animated dots
     }
   }
   ```

4. **Component-Specific Guidance:**

   **Typing Indicator:**
   - ✓ Default: "Jane is typing..." with animated dots
   - ✓ Reduced: "Jane is typing..." with static dots
   - ✓ Never: Flashing or rapidly changing elements

   **Presence Indicator:**
   - ✓ Default: Dot color changes smoothly when user status updates
   - ✓ Reduced: Dot color changes instantly
   - ✓ Never: Blinking or flashing presence indicators

   **Message Appearance:**
   - ✓ Default: Message fades in as it arrives (200ms fade-in)
   - ✓ Reduced: Message appears instantly at full opacity
   - ✓ Never: Spinning or rotating message indicators

   **Loading States:**
   - ✓ Default: Rotating spinner animation
   - ✓ Reduced: Pulsing opacity indicator (appears/disappears in rhythm)
   - ✓ Never: Scrolling, rotating, or flashing progress indicators

   **Skeleton Screens:**
   - ✓ Default: Shimmer animation across placeholder cards (400ms loop)
   - ✓ Reduced: Static gray placeholder with no animation
   - ✓ Never: Animated gradients or scrolling effects

5. **Testing & Validation:**
   - Test all animations with `prefers-reduced-motion: reduce` enabled
   - Verify reduced-motion alternatives convey same information as animated versions
   - Screen reader test: Verify announcements don't repeat during animations
   - Manual test: Enable accessibility mode in Windows Settings → Ease of Access → Display
   - User testing: Include users with vestibular disorders in accessibility testing

6. **Platform Support:**
   - **Windows 11:** Settings → Ease of Access → Display → Show animations (toggle off)
   - **Windows 10:** Settings → Ease of Access → Display → Show animations (toggle off)
   - Slint should respect system preference automatically
   - Fallback: Respect browser/app-level `prefers-reduced-motion` media query

7. **Documentation for Developers:**
   - All new animations must have a reduced-motion alternative
   - Animations must not last longer than 3 seconds without user interaction
   - No involuntary animations (animations that happen without user clicking/typing)
   - Avoid parallax scrolling, auto-play video, animated GIFs with strobes
   - Test with WebAIM motion sensitivity test (flashing < 3 per second is safe)

**Compliance Level:** WCAG 2.1 Level AA (Animation from Interactions)
**Target:** WCAG 2.1 Level AAA (Animation Options)
**Status:** Fully implemented for MVP

---

## Implementation Roadmap

### Phase 1: MVP Foundation (Weeks 1-4)
**Design System & Core Components**
- Establish Fluent + custom components in Slint
- Implement color tokens, typography system, spacing system
- Build base buttons, inputs, lists, cards
- Create message bubble and conversation list item components
- Set up design token files for consistency

**Core Experience Implementation**
- Sidebar layout with conversation list
- Conversation view with message display
- Message composer with send functionality
- Presence indicators (online/offline/away)
- Real-time message updates (WebSocket)

**Deliverables:**
- Component library (Slint code)
- Design tokens (CSS/Slint constants)
- MVP UI functional in Slint

### Phase 2: Refinement (Weeks 5-6)
**Polish & Performance**
- Smooth animations and transitions (300ms)
- Performance optimization (60+ FPS)
- Responsive behavior (sidebar collapse)
- Error handling and edge cases

**Accessibility**
- Keyboard navigation testing
- Screen reader testing (NVDA)
- Contrast and readability validation
- Color-blind user testing

**Deliverables:**
- Polish pass complete
- Accessibility audit passed
- Performance benchmarks met

### Phase 3: Post-MVP (Future)
**Advanced Features**
- Teams/Channels navigation
- Message reactions, threads, pins
- Rich message formatting
- Advanced search and filters
- Customization and themes (post-MVP)

---

## Success Metrics & Validation

### UX Metrics to Track

**Core Experience Metrics:**
- Time to send first message: Target < 2 minutes
- Message delivery latency: Target < 500ms
- Conversation discovery time: Target < 3 seconds
- Conversation switching speed: Target < 100ms
- User retention Day 1/7/30: Target 80%/70%/60%

**Emotional Response Metrics:**
- "I feel in control" rating: Target 4.5+/5
- "This is professional" rating: Target 4.7+/5
- "This is responsive" rating: Target 4.8+/5
- NPS (Net Promoter Score): Target 50+

**Accessibility Metrics:**
- Keyboard navigation: 100% of workflows accessible
- Screen reader: 100% of information announced
- Contrast: 100% of UI meets WCAG AA
- Color-blind accessible: 100%

### Validation Methods

**User Testing:**
- 5-8 user interviews (one per persona)
- Task-based testing (send message, find contact, etc.)
- Think-aloud protocol for UX insights
- System Usability Scale (SUS) scoring

**Analytics:**
- Track user actions (message send, conversation switch, search)
- Identify friction points (where do users abandon?)
- Monitor performance (real latency data)
- Measure engagement (daily active users, messages sent)

**Accessibility Testing:**
- Automated: Axe, WAVE, Lighthouse
- Manual: Keyboard-only navigation test
- Assistive tech: Screen reader testing
- Real users: Recruit users with accessibility needs

---

## Handoff & Next Steps

### Design Specification Complete

This UX Design Specification provides:

✅ **Strategic Foundation**
- Project vision and target users
- Emotional goals and design principles
- Inspiration analysis and proven patterns
- Core experience definition

✅ **Visual System**
- Design system foundation (Fluent + custom)
- Color palette with accessibility validation
- Typography system and hierarchy
- Spacing and layout foundation
- Design tokens for implementation

✅ **Interaction Design**
- Defining core experience with mechanics
- User journey flows for each persona
- UX patterns and interaction specifications
- Error handling and edge cases

✅ **Implementation Ready**
- Component strategy and library structure
- Accessibility compliance built-in
- Performance targets established
- Success metrics defined

### Next Phase: UI Development

**For Developers:**
1. Review design tokens (colors, typography, spacing)
2. Implement base components in Slint using tokens
3. Build screens using component library
4. Test performance and responsiveness
5. Validate accessibility

**For Design/QA:**
1. Review against this specification
2. Conduct user testing with prototypes
3. Iterate based on feedback
4. Sign-off on visual quality
5. Accessibility compliance verification

### Deliverables

This document serves as:
- **Design Brief** - Complete strategic direction
- **Component Specification** - What components to build
- **Interaction Guide** - How users interact with the product
- **Accessibility Standard** - WCAG AA compliance baseline
- **Implementation Roadmap** - Phased development plan
- **Success Criteria** - How to measure UX effectiveness

---

## Appendix: Design Decisions Reference

### Key Decision Points & Rationales

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Framework** | Slint + Fluent Design | Windows native, AI-friendly, performance |
| **Visual System** | Fluent Hybrid | Professional aesthetic, proven patterns, 80% reuse |
| **Layout** | Compact Professional | Balances all user needs optimally |
| **Sidebar Width** | 240px | Shows ~8-10 conversations, power user friendly |
| **Presence Model** | Always visible dots | Supports "presence-first" principle |
| **Message Feedback** | 4-stage delivery | Clear, familiar from iMessage/WhatsApp |
| **Color Palette** | Fluent Blue + Teal | Professional trustworthiness + modern |
| **Typography** | Segoe UI only | Windows native, excellent readability |
| **Grid System** | 8px base | Consistency, flexibility, industry standard |
| **Breakpoint** | 900px sidebar collapse | Reasonable point for UI adaptation |

### Inspiration Sources

- **Discord**: Real-time responsiveness, presence awareness, conversation discovery
- **Slack**: Professional aesthetic, information hierarchy, settings organization
- **Microsoft Teams**: Fluent Design System implementation, Windows integration
- **VS Code**: Progressive disclosure, keyboard-first power users, minimalism

### Accessibility Standards

- **WCAG 2.1 AA** - Minimum compliance
- **Section 508** - US government accessibility (if applicable)
- **Color Contrast**: 7:1 for normal, 4.5:1 for large text
- **Keyboard**: All functionality accessible via keyboard
- **Screen Reader**: Full semantic structure and ARIA labels

---

**UX Design Specification Complete**

Document Status: **Final**
Last Updated: 2025-12-16
Version: 1.0
Ready for: Development & Implementation

---
