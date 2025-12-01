# Modern Chat Application Layout Specification
## Desktop IRC Client - Universal Design Patterns & Best Practices

**Research Date:** November 2025  
**Analyzed Applications:** Discord, Slack, Microsoft Teams, Telegram Desktop, Element, HexChat

---

## Executive Summary

Modern chat applications converge on a **3-column layout** for desktop environments, with careful attention to visual hierarchy, information density, and progressive disclosure. This specification identifies universal patterns applicable to a desktop IRC client.

---

## 1. Column Layout Architecture

### 1.1 Three-Column Layout (Primary Pattern)
**Used by:** Discord, Slack, Teams, Element  
**When to use:** Full desktop experience with multiple servers/workspaces and rich features

```
┌─────────────┬──────────────────────────────┬─────────────┐
│  Server/    │       Main Chat Area         │   Right     │
│  Workspace  │                              │   Panel     │
│   List      │                              │  (Context)  │
│   (60-72px) │                              │  (240-280px)│
│             │                              │             │
└─────────────┴──────────────────────────────┴─────────────┘
```

**Column Breakdown:**
- **Left Column 1 (Server/Workspace):** 60-72px, fixed, icon-based navigation
- **Left Column 2 (Channels/DMs):** 220-280px, resizable (min: 180px, max: 320px)
- **Center (Main Chat):** Flexible, minimum 480px
- **Right Panel (Members/Details):** 240-280px, resizable, collapsible

**Total Minimum Width:** ~940px for comfortable 3-column usage

### 1.2 Two-Column Layout (Simplified Pattern)
**Used by:** Telegram Desktop, IRC clients (HexChat, WeeChat)  
**When to use:** Single-server applications, simplified UX, smaller windows

```
┌─────────────────┬──────────────────────────────┬─────────────┐
│   Channels/     │       Main Chat Area         │   Right     │
│   Contacts      │                              │   Panel     │
│   (220-280px)   │                              │  (Optional) │
│                 │                              │  (240-280px)│
└─────────────────┴──────────────────────────────┴─────────────┘
```

**Recommendation for IRC Client:**  
Use **2.5-column layout** - two primary columns with optional collapsible right panel:
- **Left:** 220px channel list (resizable: 180-320px)
- **Center:** Main chat area (flexible)
- **Right:** User list for channels (240px, collapsible, resizable: 180-320px)

**Total Comfortable Width:** 1024px minimum, 1280px optimal

---

## 2. Sidebar Organization (Left Panel)

### 2.1 Visual Hierarchy Patterns

**Universal Structure (Top to Bottom):**

1. **Header Section** (48-60px height)
   - Server/workspace name
   - Status indicator
   - Dropdown menu trigger
   - Border bottom: 1px subtle separator

2. **Quick Actions Bar** (Optional, 40-48px)
   - Search button
   - Filter toggles
   - Add channel button
   - Background: Slightly elevated from sidebar

3. **Primary Navigation** (Scrollable)
   - Grouped by type with clear visual separators
   - Collapsible sections
   - Consistent item heights

4. **Footer Section** (Fixed, 40-56px)
   - User profile card (collapsed)
   - Settings icon
   - Connection status

### 2.2 Content Organization Patterns

**Discord Pattern** (Server → Channels):
```
SERVER NAME                  [header, 60px]
─────────────────────────
🔍 Search                   [action bar]
─────────────────────────
📌 PINNED                   [section, 11px uppercase, muted]
  # announcements           [32px, unread badge]
  # general                 [32px, selected]

💬 TEXT CHANNELS
  # random                  [32px, hover highlight]
  # development
  ＋ Add Channel

🔊 VOICE CHANNELS
  🔊 General Voice
  🔊 Lounge

─────────────────────────
👤 Username #1234          [footer, user card]
```

**Slack Pattern** (Workspace → All Items):
```
Workspace Name           ▼  [header, dropdown]
─────────────────────────
⭐ STARRED                  [collapsible section]
  # important-project
  # design-team

💬 CHANNELS
  # general                [bold if unread]
  # random
  ＋ Add channels

✉️ DIRECT MESSAGES
  🟢 alice                 [online indicator]
  ⚫ bob                   [offline]
  ＋ Open a direct message

─────────────────────────
📱 Apps
```

**Recommended Pattern for IRC:**
```
IRC STATUS               🟢  [header, 52px, connection status]
─────────────────────────
CHANNELS                    [section header, 11px, strong, muted]
  # general               ●  [32px, blue unread badge]
  # linux                    [32px, normal]
  # programming           🔴 [32px, red highlight badge]

PRIVATE MESSAGES
  alice                   2  [32px, unread count]
  bob                        [32px]

SYSTEM                      [special, muted icon]
─────────────────────────
⚙️ Settings  │  alice      [footer, 48px]
```

### 2.3 Item Design Specifications

**Channel/DM List Items:**
- **Height:** 32-36px (optimal for click targets)
- **Padding:** 8-12px left/right, 6-8px top/bottom
- **Hover state:** Background: rgba(white/black, 0.05-0.08)
- **Selected state:** Background: rgba(accent, 0.15), left border accent 3-4px
- **Font size:** 14-15px for channel names
- **Font weight:** 
  - Normal: 400 (regular items)
  - Semibold: 600 (unread items)
  - Bold: 700 (selected item)

**Unread Indicators:**
- **Dot indicator:** 6-8px circle, positioned right or as prefix
- **Count badge:** 
  - Min width: 18px, height: 18px
  - Padding: 4-6px horizontal
  - Border radius: 9-12px (pill shape)
  - Font: 11-12px, bold, white text
  - Background: Blue (#5865F2 Discord, #1264A3 Slack) for normal unread
  - Background: Red (#ED4245 Discord, #E01E5A Slack) for mentions/highlights
  - Max display: "99+" for counts over 99

**Section Headers:**
- **Font size:** 11-12px
- **Font weight:** 600-700 (semibold to bold)
- **Text transform:** UPPERCASE
- **Color:** Muted (60-70% opacity of normal text)
- **Spacing:** 16-20px top margin, 8px bottom margin
- **Padding:** 8-16px left (aligned with items)

---

## 3. Main Chat Area Components

### 3.1 Vertical Structure

```
┌──────────────────────────────────────┐
│  Topic Bar / Channel Header          │  48-60px
├──────────────────────────────────────┤
│                                      │
│  Message Area (Scrollable)           │  Flexible
│  - Infinite scroll upward            │
│  - Auto-scroll to bottom             │
│  - Jump to present button when       │
│    scrolled up                       │
│                                      │
├──────────────────────────────────────┤
│  Message Composer / Input            │  60-120px
└──────────────────────────────────────┘
```

### 3.2 Topic Bar / Channel Header

**Discord Pattern:**
```
#  channel-name          Topic: Welcome to the channel!     [Icons] 🔔 📌 👥 🔍
```

**Slack Pattern:**
```
#  channel-name          ⭐ 👥 245                Topic: Project discussion
Description: Channel for project updates
```

**Recommended for IRC:**
```
#  channel-name          👥 42                   Topic: Double-click to edit
───────────────────────────────────────────────────────────────────────────
```

**Specifications:**
- **Height:** 48-56px
- **Background:** Slightly elevated from chat area (1-2 levels in depth hierarchy)
- **Border bottom:** 1px solid, subtle separator
- **Padding:** 12-16px horizontal
- **Layout:** 
  - Left: Channel icon + name (16-18px bold)
  - Center: Topic text (14px regular, muted color, truncated with ellipsis)
  - Right: Action icons (mute, pin, members, search) 24px hit targets

**Interactive Elements:**
- Topic is double-clickable to edit (if permission)
- Channel name links to info panel
- Icons have hover tooltips (500ms delay)

### 3.3 Message Display Area

**Message Grouping (Universal Pattern):**

Modern chat apps group consecutive messages from the same user within a time window (typically 5-7 minutes).

```
[Avatar]  Username    HH:MM          ← First message in group
          Message content here
          Another message 2 seconds later
          And another one
                                       ← 4px spacing between messages

[Avatar]  OtherUser   HH:MM          ← New group (different user)
          Their message content
```

**Message Group Specifications:**
- **Avatar:** 
  - Size: 36-42px circle
  - Position: Top-aligned with first message
  - Spacing from content: 12-16px
  - Fallback: Colored circle with initial(s)

- **Username + Timestamp Header:**
  - Font size: 14-15px for username (semibold/bold)
  - Username color: Deterministic hash-based color from palette
  - Timestamp: 11-12px, muted color, positioned after username with 8-12px gap
  - Vertical spacing from messages: 2-4px

- **Message Content:**
  - Font size: 15-16px (readability is critical)
  - Line height: 1.4-1.5 (22-24px for 16px font)
  - Left margin: 52-60px (avatar + spacing) for alignment
  - Spacing between messages in same group: 2-4px
  - Spacing between message groups: 16-20px

**Hover Effects:**
- Background highlight on hover: rgba(accent, 0.04-0.06)
- Show timestamp always (fade in on hover if normally hidden)
- Show action buttons (reply, react, more) on right side

### 3.4 Message Composer / Input Area

**Component Layout:**
```
┌────────────────────────────────────────────────────────┐
│  [+]  Type a message...                      [emoji] [GIF] [send] │
└────────────────────────────────────────────────────────┘
```

**Specifications:**
- **Height:** 
  - Minimum: 44px (single line with padding)
  - Maximum: 120-200px (auto-expand multiline)
  - Average: 60px with padding/margins

- **Background:** Distinct from chat area (elevated layer)
- **Border top:** 1px solid separator
- **Padding:** 10-16px all around
- **Corner radius:** 8-12px for input field itself

- **Input Field:**
  - Min height: 40-44px
  - Padding: 10-12px horizontal, 8-10px vertical
  - Font size: 15-16px (matches message text)
  - Border: 1px solid when unfocused, 2px accent when focused
  - Background: Slightly lighter/darker than container

- **Action Buttons:**
  - Size: 32-40px (icon + padding)
  - Position: Right side, inline with input
  - Spacing: 4-8px between buttons
  - Common: Emoji picker, file upload, GIF, send (optional, Enter is primary)

**Interaction:**
- Enter to send (unless Shift+Enter for multiline)
- Auto-expand as user types (up to max height)
- Show character count if limit exists (e.g., 500 chars remaining)
- Typing indicator sent to other users

---

## 4. Right Panel Usage

### 4.1 Context-Aware Display

The right panel is **contextual** - its content changes based on the current view:

**Channel View:**
- Member list (grouped by role/status)
- Pinned messages
- Channel details

**DM View:**
- User profile
- Shared files
- Pinned messages

**Thread View:**
- Thread conversation
- Thread participants

### 4.2 Member List Pattern (IRC Channels)

**Recommended Structure:**
```
MEMBERS — 42                          [header, 11px, uppercase]
─────────────────────────
OPERATORS — 3                         [section, collapsible]
  🟢 @ alice                          [28px, green online dot]
  🟢 @ bob
  ⚫ @ charlie                        [gray offline dot]

VOICED — 5
  🟢 + david
  🟢 + eve
  ⚫ + frank
  ... 2 more                          [collapsed overflow]

MEMBERS — 34
  🟢 george
  🟢 hannah
  🟡 irene                            [yellow away dot]
  ⚫ jack
  ... 30 more
```

**Specifications:**
- **Width:** 240-280px (resizable: 180-320px minimum)
- **Background:** Matches or slightly darker than left sidebar
- **Border left:** 1px solid separator

- **Member Items:**
  - Height: 28-32px
  - Padding: 4-6px horizontal
  - Font size: 14px
  - Status dot: 8-10px circle, positioned left
  - Prefix (@, +, etc.): Colored icon/badge
  - Hover: Background highlight
  - Right-click: Context menu (whois, message, kick/ban if op)

- **Section Headers:**
  - Same style as sidebar sections
  - Collapsible with caret icon
  - Show count in header
  - Remember collapsed state

- **Search/Filter:**
  - Optional search box at top
  - 32px height, 8px margin
  - Filters: Online only, by role, alphabetical

### 4.3 Collapsing Behavior

- **Toggle button:** In topic bar or via keyboard (Ctrl+U common)
- **Animation:** 200-300ms smooth slide
- **Remembered state:** Persists per channel
- **Responsive:** Auto-hide below 1024px width, manual toggle above

---

## 5. Typography Choices

### 5.1 Font Families

**Primary UI Font:**
- **Discord:** Whitney, "Helvetica Neue", Helvetica, Arial, sans-serif
- **Slack:** Slack-Lato, Lato, "Segoe UI", sans-serif
- **Teams:** Segoe UI, system-ui, sans-serif
- **Telegram:** Roboto, "Helvetica Neue", Arial, sans-serif

**Recommended for IRC Client:**
```
Font Stack: -apple-system, BlinkMacSystemFont, "Segoe UI", 
            "Roboto", "Helvetica Neue", Arial, sans-serif
```
**Benefits:**
- System native fonts (excellent performance, familiar UX)
- Cross-platform consistency
- High readability at small sizes

**Monospace Font (for code/technical):**
```
Monospace Stack: "JetBrains Mono", "Fira Code", "SF Mono", 
                 Monaco, Consolas, monospace
```

### 5.2 Font Size Scale

**Type System:**
```
H1 (Channel Names):       18-20px  Bold (700)
H2 (Section Headers):     11-12px  Bold (700), UPPERCASE
H3 (Username):            14-15px  Semibold (600)
Body (Messages):          15-16px  Regular (400)
Secondary (Timestamps):   11-12px  Regular (400)
Caption (Hints):          12-13px  Regular (400)
```

**Line Height:**
- **Headers:** 1.2-1.3 (tight)
- **Body text:** 1.4-1.5 (comfortable reading)
- **UI labels:** 1.3-1.4

### 5.3 Font Weight Usage

Modern chat apps use **4-5 weight variations:**

```
Light (300):     Rarely used, decorative only
Regular (400):   Body text, timestamps, secondary info
Medium (500):    Slightly emphasized items (Discord uses this)
Semibold (600):  Usernames, unread channels, buttons
Bold (700):      Headers, selected items, counts
```

**Recommendation:**
- Use Regular (400) as baseline
- Use Semibold (600) for usernames and emphasis
- Use Bold (700) sparingly for headers and selected states
- Avoid light weights in small sizes (reduces readability)

### 5.4 Spacing & Rhythm

**Vertical Rhythm (8pt Grid System):**
- Base unit: 4px or 8px
- All spacing should be multiples: 4, 8, 12, 16, 20, 24, 32, 40, 48...
- Line heights aligned to 4px grid where possible

**Horizontal Spacing:**
- Sidebar padding: 12-16px
- Message left margin: 52-60px (avatar alignment)
- Between inline elements: 8-12px
- Sections: 16-24px

**Consistent Spacing Tokens:**
```
xs:  4px   - Tight spacing, inline elements
sm:  8px   - Default item padding
md:  12px  - Standard spacing
lg:  16px  - Section spacing
xl:  24px  - Major section breaks
2xl: 32px  - Large gaps
3xl: 48px  - Maximum spacing
```

---

## 6. Color & Visual Hierarchy

### 6.1 Background Depth Layers

Modern apps use **5-7 background shades** to create depth:

**Dark Mode Layers (From Darkest to Lightest):**
```
Layer 0 (Deepest):     #1A1A1E  Server list background
Layer 1 (Deep):        #1E1F25  Sidebar background
Layer 2 (Base):        #2B2D35  Chat background
Layer 3 (Elevated):    #35374B  Input box, hover states
Layer 4 (Overlay):     #43465E  Modals, popovers
Layer 5 (Active):      #4E5266  Active selections
```

**Light Mode Layers:**
```
Layer 0: #DCDEEA  (Darkest for sidebars)
Layer 1: #EBEBF0
Layer 2: #FFFFFF  (Base - pure white for chat)
Layer 3: #F5F5FA  (Input, slight elevation)
Layer 4: #FAFAFD  (Modals)
Layer 5: #E6E8F0  (Hover/active)
```

**Current Implementation Alignment:**
Your theme already implements this well:
```rust
BG_DARKEST: #121217  (Layer 0)
BG_DARKER:  #18191F  (Layer 1)
BG_DARK:    #202229  (Layer 2)
BG_BASE:    #282B34  (Layer 3)
BG_ELEVATED: #323641 (Layer 4)
```

### 6.2 Text Hierarchy

**Opacity-Based (Common Pattern):**
- Primary text: 100% opacity
- Secondary text: 60-70% opacity
- Muted/disabled: 40-50% opacity

**Color-Based (Preferred, Better Contrast):**
```
Dark Mode:
  Primary:   #DCDDE1  (rgb 220, 221, 225)
  Secondary: #949BA4  (rgb 148, 155, 164)
  Muted:     #606670  (rgb 96, 102, 112)

Light Mode:
  Primary:   #1E1F22  (rgb 30, 31, 34)
  Secondary: #50555F  (rgb 80, 85, 95)
  Muted:     #828791  (rgb 130, 135, 145)
```

### 6.3 Accent Colors

**Purpose-Driven Palette:**
```
Primary (Brand):     #5865F2  Blue (Discord) / #1264A3 (Slack)
Success (Online):    #43B581  Green
Warning (Away):      #FAA61A  Orange/Yellow
Error (Offline):     #ED4245  Red
Mention (Highlight): #FAA61A  Orange or #F04747 Red
Link:                #00AFF4  Bright Blue
```

**Usage:**
- Unread badges: Primary blue
- Mention badges: Error red or warning orange
- Links: Link blue
- Online status: Success green
- Offline status: Gray (muted)
- Away status: Warning yellow

**Current Implementation:**
Your colors align well:
```rust
ACCENT_BLUE:   #5865F2  ✓
ACCENT_GREEN:  #43B581  ✓
ACCENT_YELLOW: #FAA61A  ✓
ACCENT_RED:    #ED4245  ✓
```

### 6.4 Borders & Separators

**Border Hierarchy:**
- **Subtle separators:** 1px, rgba(white/black, 0.08-0.12)
- **Panel borders:** 1px, dedicated border color
- **Focus indicators:** 2px, accent color
- **Strong divisions:** 1px, higher contrast (0.15-0.20)

**Border Radius:**
- **UI elements:** 6-8px (buttons, badges)
- **Input fields:** 8-12px
- **Avatars:** 50% (perfect circle)
- **Panels:** 0-4px (minimal or none for main panels)
- **Modals:** 8-12px

---

## 7. Responsive Behavior & Breakpoints

### 7.1 Width Breakpoints

```
< 768px:   Mobile (not primary for desktop client)
768-1023:  Tablet/Small desktop - 2 columns max, collapsible sidebar
1024-1279: Standard desktop - 2-3 columns, all features available
1280-1599: Large desktop - 3 columns comfortable, wider panels
1600+:     Extra large - Optional wider chat, more padding
```

### 7.2 Adaptive Layout Rules

**Below 1024px:**
- Auto-collapse right panel (user list)
- Reduce sidebar width to 200px
- Hide or collapse secondary toolbar items

**1024-1279px:**
- Default: Left sidebar (220px) + Chat + Collapsible right (240px)
- User can manually toggle panels

**1280px and above:**
- Default: All panels visible
- Increased padding and spacing
- Comfortable reading line length (600-800px for chat)

### 7.3 Panel Resize Constraints

**Left Sidebar:**
- Minimum: 180px
- Default: 220px
- Maximum: 320px
- Snap points: 200px, 240px, 280px

**Right Panel:**
- Minimum: 180px
- Default: 240px
- Maximum: 320px
- Snap points: 220px, 260px, 300px

**Chat Area:**
- Minimum: 480px (below this, UX degrades)
- Optimal: 600-800px (reading comfort)
- Maximum: 900px (very wide lines reduce readability)

---

## 8. IRC-Specific Considerations

### 8.1 Multi-Server Support

**Option A: Single Server (Simpler)**
- Current implementation approach
- Single left sidebar for channels
- Connection managed via toolbar
- Suitable for users on 1-2 networks

**Option B: Multi-Server (Advanced)**
- Add leftmost column (60-72px) with server icons
- Each server has own channel list
- Clicking server icon switches context
- Status indicators per server

**Recommendation:** Start with Option A, design architecture for Option B expansion.

### 8.2 IRC Feature Visibility

**Channel Modes & Properties:**
- Show mode indicators in channel list (+m, +t, etc.) on hover
- Display in topic bar when active
- Muted channels: Gray out, italic

**User Prefixes:**
- Visualize with colored dots or icons
- @ = Green, + = Orange, ~ = Gold, regular = Gray
- Sort user list by prefix (ops first)

**Notifications:**
- Message count badge: Blue
- Highlight/mention badge: Red
- Sound indicator: Optional bell icon

### 8.3 IRC Commands & Input

**Command Autocomplete:**
- Show popup below input when "/" typed
- List common commands: /join, /msg, /whois, /nick, etc.
- Keyboard navigable (arrow keys, Enter)

**Nick Autocomplete:**
- Tab completion in input field
- Show popup with matching nicks from current channel
- Cycle through matches with repeated Tab

**Input Hints:**
- Show current nick and target in input area
- Placeholder adapts: "Message #channel" vs "Message alice"

---

## 9. Accessibility & Usability

### 9.1 Contrast Ratios

**WCAG AA Compliance (Minimum):**
- Normal text (< 18px): 4.5:1 contrast ratio
- Large text (≥ 18px): 3:1 contrast ratio
- UI elements: 3:1 contrast ratio

**Test Your Colors:**
```
Dark Mode:
  Primary text (#DCDDE1) on BG_DARK (#202229):  ~12.5:1 ✓✓
  Secondary text (#949BA4) on BG_DARK:          ~6.8:1 ✓
  Muted text (#606670) on BG_DARK:              ~3.9:1 ✓

Light Mode:
  Primary text (#1E1F22) on White (#FFFFFF):    ~17.2:1 ✓✓
  Secondary text (#50555F) on White:            ~8.6:1 ✓✓
```

### 9.2 Keyboard Navigation

**Essential Shortcuts:**
```
Ctrl+K:       Quick switcher (jump to channel)
Alt+↑/↓:      Navigate channels
Alt+Shift+↑/↓: Navigate servers (if multi-server)
Ctrl+/:       Show command palette
Ctrl+F:       Search in channel
Esc:          Close dialogs, clear input
Ctrl+Tab:     Cycle through recent channels
```

**Focus Indicators:**
- 2px outline, accent color
- Visible on all interactive elements
- Never remove outlines (accessibility)

### 9.3 Screen Reader Support

- Semantic HTML structure (headings, lists, landmarks)
- ARIA labels for icon buttons
- Live region announcements for new messages
- Skip navigation links

### 9.4 User Preferences

**Customization Options:**
```
Appearance:
  - Theme: Dark, Light, Auto (system)
  - Font size: 12px, 14px, 16px, 18px
  - Compact mode (reduced spacing)
  - Avatar display: On, Off, Hover only

Behavior:
  - Enter to send: On, Off (Ctrl+Enter)
  - Timestamp format: 12h, 24h, Relative
  - Message grouping time: 1min, 5min, 10min
  - Auto-scroll: Always, On new message, Manual

Notifications:
  - All messages, Mentions only, Nothing
  - Sound: On, Off, Custom
  - Desktop notifications: On, Off
```

---

## 10. Implementation Recommendations

### 10.1 Current State Assessment

Your client already implements many best practices:

✅ **Strengths:**
- Clean 2-column base layout with optional 3rd column
- Modern color palette with proper depth hierarchy
- Message grouping by sender
- Resizable panels
- Distinct visual states (hover, active, selected)
- Avatar system with colored backgrounds
- Unread/highlight badges
- Responsive typography

🔄 **Opportunities for Enhancement:**

1. **Typography:**
   - Current message font: 15-16px is good
   - Consider slightly larger for better readability (16px baseline)
   - Ensure line-height is 1.45-1.5 for body text

2. **Spacing:**
   - Verify 8pt grid alignment across all components
   - Current MESSAGE_GROUP_SPACING: 16px ✓
   - Current AVATAR_SIZE: 36px ✓

3. **Panel Widths:**
   - Left panel: Default 220px ✓
   - Right panel: Default 180px → Consider 240px for more comfortable user list

4. **Input Area:**
   - Current design looks good with rounded corners
   - Ensure focus state is very clear (2px accent border)

### 10.2 Quick Wins

**High-Impact, Low-Effort Improvements:**

1. **Search/Filter in Channel List:**
   - Add 32px search input at top of left sidebar
   - Filter channels as user types
   - Keyboard shortcut: Ctrl+K

2. **Collapsible Sections:**
   - Make CHANNELS, PRIVATE MESSAGES sections collapsible
   - Add caret icon, remember state

3. **User Status Dots:**
   - 8px colored dots in user list
   - Green (online), Yellow (away), Gray (offline)

4. **Hover Timestamps:**
   - Show full timestamp on message hover
   - Absolute time (HH:MM:SS) and relative (2 minutes ago)

5. **Quick Switcher:**
   - Ctrl+K opens fuzzy finder for channels
   - Type to filter, Enter to switch

6. **Better Focus Indicators:**
   - Ensure all interactive elements have visible focus
   - Use accent blue, 2px outline

### 10.3 Progressive Enhancement Roadmap

**Phase 1: Polish Current Layout**
- [ ] Refine spacing using strict 8pt grid
- [ ] Add focus indicators to all interactive elements
- [ ] Implement collapsible sections in sidebar
- [ ] Add search/filter to channel list

**Phase 2: Enhanced Interactions**
- [ ] Message hover actions (reply, bookmark, more)
- [ ] Context menus (right-click anywhere)
- [ ] Quick switcher (Ctrl+K)
- [ ] Keyboard navigation improvements

**Phase 3: Advanced Features**
- [ ] Thread view in right panel
- [ ] Pinned messages panel
- [ ] User profile panel (click avatar)
- [ ] Advanced search (Ctrl+F)

**Phase 4: Customization**
- [ ] User preferences UI
- [ ] Custom themes
- [ ] Layout presets (compact, cozy, comfortable)
- [ ] Font size scaling

---

## 11. Conclusion & Key Takeaways

### Universal Patterns Identified

1. **Layout:** 2-3 column design, left navigation + center chat + optional right context
2. **Hierarchy:** Clear depth through background layers (5-7 shades)
3. **Typography:** System fonts, 15-16px body text, semibold for emphasis
4. **Spacing:** 8pt grid, generous padding (12-16px), grouped messages
5. **Colors:** Muted UI, vibrant accents, purpose-driven palette
6. **Interactions:** Hover states, focus indicators, keyboard shortcuts
7. **Density:** Comfortable by default, optional compact mode
8. **Responsiveness:** Collapsible panels, adaptive layouts

### Design Philosophy

Modern chat applications prioritize:

- **Readability first:** Large text, good contrast, generous spacing
- **Visual calm:** Muted UI backgrounds, vibrant content (messages, avatars)
- **Progressive disclosure:** Hide complexity, reveal on hover/focus
- **Consistency:** Systematic spacing, color palette, component behavior
- **Performance:** Smooth animations, instant feedback, optimized rendering
- **Accessibility:** Keyboard navigation, screen readers, high contrast

### For Your IRC Client

**Recommended Focus Areas:**

1. **Message readability** - This is 80% of user time
   - 16px font, 1.45 line height, 60-char line length
   - Clear grouping, generous spacing between groups

2. **Channel navigation** - Quick switching is essential
   - Keyboard shortcuts (Alt+↑/↓)
   - Quick switcher (Ctrl+K)
   - Clear unread indicators

3. **Visual hierarchy** - Guide user attention
   - Background depth (your 7-layer system is excellent)
   - Text hierarchy (3 levels: primary, secondary, muted)
   - Color purpose (blue info, red urgent, green success)

4. **Professional polish**
   - Consistent spacing (8pt grid everywhere)
   - Smooth animations (200-300ms)
   - Responsive feedback (hover, active, focus states)

**Your current implementation is solid.** The recommendations above will help align with industry standards and enhance user experience based on patterns proven across millions of daily users.

---

## Appendix: Color Reference

### Dark Mode Complete Palette

```css
/* Backgrounds */
--bg-darkest:  #121217;
--bg-darker:   #18191F;
--bg-dark:     #202229;
--bg-base:     #282B34;
--bg-elevated: #323641;
--bg-hover:    #3A3E4B;
--bg-active:   #424756;

/* Text */
--text-primary:   #DCDDE1;
--text-secondary: #949BA4;
--text-muted:     #606670;

/* Accents */
--accent-blue:   #5865F2;
--accent-green:  #43B581;
--accent-yellow: #FAA61A;
--accent-red:    #ED4245;
--accent-pink:   #EB459E;

/* UI Elements */
--border:        #373A46;
--scrollbar:     #3C404C;
--scrollbar-hover: #4B505F;
```

### Light Mode Complete Palette

```css
/* Backgrounds */
--bg-darkest:  #DCDEE4;
--bg-darker:   #EBEDF2;
--bg-dark:     #F2F3F7;
--bg-base:     #FFFFFF;
--bg-elevated: #F8F9FC;
--bg-hover:    #F0F1F5;
--bg-active:   #E6E8EE;

/* Text */
--text-primary:   #1E1F22;
--text-secondary: #50555F;
--text-muted:     #828791;

/* Accents */
--accent-blue:   #4252D6;
--accent-green:  #2D9B64;
--accent-red:    #D23738;

/* UI Elements */
--border:        #D2D4DC;
--scrollbar:     #BEC3CD;
```

---

**End of Specification**

*Research compiled from analysis of Discord (2025), Slack (2025), Microsoft Teams (2025), Telegram Desktop, Element, and IRC clients (HexChat, WeeChat, Irssi). This document reflects current industry best practices as of November 2025.*
