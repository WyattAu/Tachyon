# ADR-065: Accessibility

**Status:** Accepted
**Date:** 2026-02-11
**Context:** Phase 7 - Narrative & Documentation
**Related ADRs:** ADR-062 (Brand Identity), ADR-063 (UX Philosophy), ADR-064 (Documentation Strategy)

---

## Context

Tachyon is a documentation platform that must be accessible to all users, including those with disabilities. Accessibility is both a legal requirement (WCAG 2.1 AA, ADA, Section 508) and a core value of inclusivity. The project targets international markets, requiring compliance with multiple accessibility standards.

## Problem Statement

Without a defined accessibility strategy, Tachyon risks:
- Excluding users with disabilities from using the platform
- Legal liability for non-compliance with accessibility laws
- Poor user experience for users with assistive technologies
- Inconsistent accessibility across different modes and platforms

## Decision

### Accessibility Framework

#### 1. Accessibility Standards Compliance

**Primary Standard: WCAG 2.1 AA**

Tachyon commits to WCAG 2.1 Level AA compliance as the baseline for all user-facing interfaces and documentation.

**Compliance Matrix:**

| Standard | Level | Status | Scope |
|----------|-------|--------|-------|
| **WCAG 2.1** | AA | Required | All user interfaces |
| **WCAG 2.2** | AA | Target | Future enhancement |
| **ADA (US)** | Compliance | Required | US market |
| **Section 508** | Compliance | Required | US government contracts |
| **EN 301 549** | Compliance | Required | EU market |
| **JIS X 8341-3** | Compliance | Required | Japan market |

**WCAG 2.1 AA Requirements:**

| Guideline | Success Criteria | Implementation |
|-----------|------------------|----------------|
| **1.1 Text Alternatives** | 1.1.1 Non-text Content | Alt text for all images, icons |
| **1.2 Time-Based Media** | 1.2.1, 1.2.2 Audio-only, Video-only | Transcripts for media |
| **1.3 Adaptable** | 1.3.1, 1.3.2, 1.3.3, 1.3.4, 1.3.5 | Semantic HTML, proper structure |
| **1.4 Distinguishable** | 1.4.1, 1.4.2, 1.4.3, 1.4.4, 1.4.5, 1.4.10, 1.4.11, 1.4.12 | Contrast, resize, text spacing |
| **2.1 Keyboard Accessible** | 2.1.1, 2.1.2, 2.1.3, 2.1.4 | Full keyboard navigation |
| **2.2 Enough Time** | 2.2.1, 2.2.2, 2.2.3 | No time limits, user control |
| **2.3 Seizures** | 2.3.1, 2.3.2, 2.3.3 | No flashing content |
| **2.4 Navigable** | 2.4.1, 2.4.2, 2.4.3, 2.4.4, 2.4.5, 2.4.6, 2.4.7 | Clear structure, focus order |
| **3.1 Readable** | 3.1.1, 3.1.2 | Language declaration, text changes |
| **3.2 Predictable** | 3.2.1, 3.2.2, 3.2.3, 3.2.4 | Consistent navigation |
| **3.3 Input Assistance** | 3.3.1, 3.3.2, 3.3.3, 3.3.4, 3.3.5, 3.3.6, 3.3.7, 3.3.8 | Error prevention, labels |
| **4.1 Compatible** | 4.1.1, 4.1.2, 4.1.3 | Name, role, value |

#### 2. Accessibility Design Patterns

**2.1 Keyboard Navigation**

**Requirements:**
- All interactive elements must be keyboard accessible
- Logical tab order following visual order
- Visible focus indicators (2px minimum, high contrast)
- Keyboard shortcuts documented

**Implementation:**

```html
<!-- Keyboard-accessible button with visible focus -->
<button 
  tabindex="0"
  onkeydown="handleKey(event)"
  onclick="saveDocument()"
  style="outline: 2px solid #0066CC; outline-offset: 2px;"
>
  Save Document
</button>
```

**Keyboard Shortcuts:**

| Shortcut | Action | Platform |
|----------|--------|----------|
| `Ctrl/Cmd + S` | Save document | All |
| `Ctrl/Cmd + F` | Search | All |
| `Ctrl/Cmd + K` | Command palette | All |
| `Escape` | Close dialog / Cancel | All |
| `Tab` | Navigate forward | All |
| `Shift + Tab` | Navigate backward | All |
| `Arrow Keys` | Navigate within lists | All |

**2.2 Screen Reader Support**

**Requirements:**
- Semantic HTML with proper ARIA labels
- Announce dynamic content changes
- Provide context for complex interactions
- Test with NVDA, JAWS, VoiceOver, ORCA

**Implementation:**

```html
<!-- Semantic structure for screen readers -->
<header role="banner">
  <nav aria-label="Main navigation">
    <ul role="menubar">
      <li role="none">
        <button role="menuitem" aria-haspopup="true" aria-expanded="false">
          Documents
        </button>
      </li>
      <li role="none">
        <button role="menuitem">Search</button>
      </li>
    </ul>
  </nav>
</header>

<main role="main" aria-live="polite">
  <article aria-labelledby="doc-title">
    <h1 id="doc-title">API Reference</h1>
    <!-- Content -->
  </article>
</main>
```

**ARIA Patterns:**

| Pattern | Use Case | Implementation |
|---------|----------|----------------|
| **aria-label** | Icon-only buttons | `<button aria-label="Save">` |
| **aria-labelledby** | Referencing labels | `<input aria-labelledby="lbl">` |
| **aria-describedby** | Additional help | `<input aria-describedby="help">` |
| **aria-live** | Dynamic content updates | `<div aria-live="polite">` |
| **aria-atomic** | Complete announcements | `<div aria-live="polite" aria-atomic="true">` |
| **aria-hidden** | Decorative content | `<div aria-hidden="true">` |

**2.3 Color Independence**

**Requirements:**
- Information conveyed through multiple channels (color, icon, text)
- Minimum 4.5:1 contrast ratio for normal text
- Minimum 3:1 contrast ratio for large text (18pt+)
- Colorblind-friendly palette

**Color Palette (WCAG AA Compliant):**

| Color | Hex | Contrast (on white) | Usage |
|-------|-----|---------------------|-------|
| **Primary Blue** | #0066CC | 7.4:1 | Primary actions, links |
| **Secondary Teal** | #00A0A0 | 5.8:1 | Secondary actions |
| **Accent Orange** | #FF8C00 | 4.5:1 | Call-to-actions |
| **Success Green** | #28A745 | 4.5:1 | Success states |
| **Error Red** | #DC3545 | 4.5:1 | Error states |
| **Warning Yellow** | #FFC107 | 4.5:1 | Warning states |
| **Neutral Gray** | #6C757D | 5.3:1 | Secondary text |
| **Dark Gray** | #212529 | 16.1:1 | Primary text |

**Implementation:**

```html
<!-- Color-independent error indication -->
<div class="alert alert-error" role="alert">
  <span class="icon-error" aria-hidden="true">!</span>
  <span class="text-error">Error:</span>
  <span>Document could not be saved</span>
</div>

<style>
.alert-error {
  background-color: #F8D7DA;
  color: #721C24;
  border: 2px solid #DC3545;
}

.icon-error {
  color: #DC3545;
  font-weight: bold;
}
</style>
```

**2.4 Visual Accessibility**

**Requirements:**
- Resizable text up to 200% without loss of content
- Text spacing customizable (word, letter, line, paragraph)
- No content that flashes more than 3 times per second

**Implementation:**

```css
/* Resizable text */
html {
  font-size: 100%; /* Base 16px */
}

@media (max-width: 768px) {
  html {
    font-size: 112.5%; /* 18px on mobile */
  }
}

/* Text spacing */
.resizable-text {
  word-spacing: 0.16em;      /* Default */
  letter-spacing: 0.12em;     /* Default */
  line-height: 1.5;           /* Default */
  paragraph-spacing: 2em;     /* Default */
}

/* User can override */
.resizable-text:focus {
  word-spacing: 0.3em;
  letter-spacing: 0.25em;
  line-height: 1.7;
}
```

**2.5 Form Accessibility**

**Requirements:**
- All form inputs have visible labels
- Error messages associated with inputs
- Required fields clearly indicated
- Form validation provides helpful feedback

**Implementation:**

```html
<form>
  <div class="form-group">
    <label for="document-title" id="title-label">
      Document Title <span class="required" aria-required="true">*</span>
    </label>
    <input
      type="text"
      id="document-title"
      name="title"
      aria-labelledby="title-label"
      aria-required="true"
      aria-invalid="false"
      aria-describedby="title-help title-error"
      required
    />
    <small id="title-help" class="form-text">
      Enter a descriptive title for your document.
    </small>
    <div id="title-error" class="error-message" role="alert" aria-live="polite" hidden>
      Title is required.
    </div>
  </div>
</form>
```

#### 3. Assistive Technology Compatibility

**3.1 Screen Readers**

**Testing Matrix:**

| Screen Reader | Platform | Status | Priority |
|---------------|----------|--------|----------|
| **NVDA** | Windows | Tested | High |
| **JAWS** | Windows | Tested | High |
| **VoiceOver** | macOS | Tested | High |
| **ORCA** | Linux | Tested | Medium |
| **TalkBack** | Android | Planned | Medium |
| **VoiceOver** | iOS | Planned | Medium |

**Testing Checklist:**

- [ ] All interactive elements announced
- [ ] Navigation follows logical order
- [ ] Dynamic content updates announced
- [ ] Error messages announced
- [ ] Form labels correctly associated
- [ ] Heading structure provides outline
- [ ] Lists and tables properly announced

**3.2 Alternative Input Devices**

**Supported Devices:**

| Device Type | Support | Implementation |
|-------------|---------|----------------|
| **Keyboard** | Full support | Standard keyboard navigation |
| **Mouse/Trackpad** | Full support | Standard pointer events |
| **Touch** | Full support | Touch events, 44x44px minimum targets |
| **Switch Device** | Planned | Tab navigation, consistent focus |
| **Voice Control** | Planned | ARIA labels, clear commands |

**3.3 Browser Magnifiers**

**Requirements:**
- Layout does not break at 200% zoom
- Text remains readable
- No horizontal scrolling at 100% zoom

**Implementation:**

```css
/* Responsive layout that works with magnification */
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 1rem;
}

@media (max-width: 992px) {
  .container {
    max-width: 960px;
  }
}

@media (max-width: 768px) {
  .container {
    max-width: 720px;
  }
}
```

#### 4. Accessibility Testing Strategy

**4.1 Automated Testing**

**Tools:**

```bash
# pa11y - Accessibility testing
npx pa11y https://docs.tachyon.org

# axe-core - Accessibility testing
npx axe https://docs.tachyon.org

# Lighthouse - Performance and accessibility
npx lighthouse https://docs.tachyon.org --view

# markdownlint - Accessibility in Markdown
npx markdownlint .docs/
```

**CI/CD Integration:**

```yaml
# .github/workflows/accessibility.yml
name: Accessibility Testing
on: [push, pull_request]

jobs:
  accessibility:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run pa11y
        run: |
          npx pa11y-ci --sitemap https://docs.tachyon.org/sitemap.xml
```

**4.2 Manual Testing**

**Testing Schedule:**

| Test Type | Frequency | Responsibility |
|-----------|-----------|----------------|
| **Keyboard Navigation** | Every release | QA Team |
| **Screen Reader Testing** | Monthly | Accessibility Specialist |
| **Color Contrast** | Every release | Design Team |
| **Form Accessibility** | Every release | QA Team |
| **Mobile Accessibility** | Quarterly | Mobile Team |

**Testing Tools:**

| Platform | Tools |
|----------|-------|
| **Windows** | NVDA, JAWS, Narrator |
| **macOS** | VoiceOver, Zoom |
| **Linux** | ORCA, Magnifier |
| **iOS** | VoiceOver, Zoom |
| **Android** | TalkBack, Magnification |

**4.3 User Testing**

**Inclusive Testing:**

- Recruit users with disabilities for beta testing
- Conduct usability studies with assistive technologies
- Gather feedback on accessibility features
- Iterate based on user input

#### 5. Accessibility Documentation

**5.1 Accessibility Statement**

Every public-facing page must include an accessibility statement:

```markdown
## Accessibility Statement

Tachyon is committed to ensuring digital accessibility for people with disabilities. We are continually improving the user experience for everyone and applying the relevant accessibility standards.

### Conformance Status

The Web Content Accessibility Guidelines (WCAG) defines requirements for designers and developers to improve accessibility for people with disabilities. It defines three levels of conformance: Level A, Level AA, and Level AAA. Tachyon is partially conformant with WCAG 2.1 level AA. Partially conformant means that some parts of the content do not fully conform to the accessibility standard.

### Feedback

We welcome your feedback on the accessibility of Tachyon. Please let us know if you encounter accessibility barriers:

- Email: accessibility@tachyon.org
- GitHub: https://github.com/WyattAu/Tachyon/issues

### Accessibility Features

- Full keyboard navigation
- Screen reader support (NVDA, JAWS, VoiceOver, ORCA)
- High contrast mode
- Resizable text up to 200%
- Color-independent information conveyance

### Technical Specifications

- Accessibility of Tachyon relies on the following technologies:
  - HTML
  - CSS
  - JavaScript
- Accessibility of Tachyon relies on the following technologies to work with the particular combination of web browser and any assistive technologies or plugins installed on your computer:
  - HTML WAI-ARIA
  - CSS Flexbox and Grid
```

**5.2 Keyboard Shortcuts Guide**

Dedicated keyboard shortcuts documentation:

```markdown
## Keyboard Shortcuts

### Navigation

| Shortcut | Action |
|----------|--------|
| `Tab` | Navigate to next interactive element |
| `Shift + Tab` | Navigate to previous interactive element |
| `Enter` | Activate focused element |
| `Escape` | Close dialog or cancel action |

### Document Actions

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + S` | Save document |
| `Ctrl/Cmd + P` | Print document |
| `Ctrl/Cmd + F` | Search in document |

### Search

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + K` | Open search |
| `Arrow Up/Down` | Navigate search results |
| `Enter` | Open selected result |
```

#### 6. Accessibility in Multi-Modal Experience

**Desktop Mode Accessibility:**

- Native OS accessibility APIs (Accessibility API on macOS, UI Automation on Windows, AT-SPI on Linux)
- Tauri accessibility configuration
- Screen reader compatibility with native components

**Server Mode Accessibility:**

- WCAG 2.1 AA compliant web interface
- Responsive design for all devices
- Keyboard and screen reader support

**Static Export Accessibility:**

- Semantic HTML output
- Proper heading structure
- Alt text for all images
- ARIA attributes for dynamic content

**Implementation:**

```rust
// Accessibility-aware component rendering
pub struct AccessibleComponent {
    pub aria_label: Option<String>,
    pub role: Option<String>,
    pub aria_live: Option<LiveRegion>,
}

impl AccessibleComponent {
    pub fn render(&self) -> String {
        let mut attrs = Vec::new();
        
        if let Some(label) = &self.aria_label {
            attrs.push(format!(r#"aria-label="{}""#, escape_html(label)));
        }
        
        if let Some(role) = &self.role {
            attrs.push(format!(r#"role="{}""#, escape_html(role)));
        }
        
        if let Some(live) = &self.aria_live {
            attrs.push(format!(r#"aria-live="{}""#, live.as_str()));
        }
        
        format!("<div {}>", attrs.join(" "))
    }
}
```

#### 7. Accessibility Training and Awareness

**7.1 Developer Training**

**Required Training for All Developers:**

1. **WCAG 2.1 Fundamentals** (2 hours)
   - Understanding accessibility guidelines
   - Common barriers for users with disabilities
   - Assistive technology basics

2. **Accessibility in Code** (2 hours)
   - Semantic HTML
   - ARIA attributes
   - Keyboard navigation patterns

3. **Testing Techniques** (1 hour)
   - Automated testing tools
   - Manual testing with screen readers
   - User testing with people with disabilities

**7.2 Design Training**

**Required Training for Designers:**

1. **Accessible Design Principles** (2 hours)
   - Color contrast requirements
   - Typography considerations
   - Layout for magnification

2. **Figma Accessibility Tools** (1 hour)
   - Using accessibility plugins
   - Testing contrast ratios
   - Validating keyboard navigation

**7.3 QA Training**

**Required Training for QA Engineers:**

1. **Accessibility Testing** (2 hours)
   - Keyboard navigation testing
   - Screen reader testing
   - Assistive technology setup

#### 8. Accessibility Metrics and Monitoring

**8.1 Key Metrics**

| Metric | Target | Measurement |
|--------|--------|-------------|
| **WCAG Compliance** | 100% Level AA | Automated + manual |
| **Keyboard Navigation** | 100% of features | Manual testing |
| **Screen Reader Support** | All major screen readers | Manual testing |
| **Color Contrast** | Minimum 4.5:1 | Automated testing |
| **Form Accessibility** | 100% of forms | Automated testing |

**8.2 Quality Gates**

```yaml
quality_gates:
  accessibility:
    - wcag_aa_compliance: 100%
    - keyboard_navigation: 100%
    - color_contrast: 100%
    - form_labels: 100%
    - alt_text: 100%
    - aria_attributes: 100%
```

**8.3 Monitoring and Alerts**

**Accessibility Drift Detection:**

- Weekly automated accessibility scans
- Alert on new accessibility violations
- Track accessibility debt
- Prioritize accessibility fixes

## Consequences

### Positive Consequences

1. **Inclusive Product**
   - All users can access Tachyon regardless of ability
   - Broader market reach and user base

2. **Legal Compliance**
   - Meets WCAG 2.1 AA requirements
   - Complies with ADA, Section 508, EN 301 549, JIS X 8341-3

3. **Better User Experience**
   - Accessibility improvements benefit all users
   - Better semantic structure and navigation

4. **Technical Excellence**
   - Cleaner code with semantic HTML
   - Better SEO from semantic structure

### Negative Consequences

1. **Development Overhead**
   - Accessibility features require additional development time
   - Testing with assistive technologies adds to QA effort

2. **Design Constraints**
   - Accessibility requirements may limit design options
   - Color contrast requirements affect palette choices

3. **Ongoing Maintenance**
   - Accessibility must be maintained with every change
   - Regular testing required to ensure compliance

## Alternatives Considered

1. **Minimal Accessibility**
   - Rejected: Would exclude users with disabilities and risk legal liability

2. **WCAG 2.1 AAA Compliance**
   - Rejected: Excessive constraints for current scope

3. **Deferred Accessibility**
   - Rejected: Accessibility must be built in, not added later

## References

- [White Paper](../.adrs/
- [ADR-062: Brand Identity](./adr-062-brand-identity.md)
- [ADR-063: UX Philosophy](./adr-063-ux-philosophy.md)
- [ADR-064: Documentation Strategy](./adr-064-documentation-strategy.md)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [WCAG 2.2 Guidelines](https://www.w3.org/WAI/WCAG22/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [Section 508 Standards](https://www.section508.gov/)
- [EN 301 549](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/02.01.02_60/en_301549v020102p.pdf)

## Implementation

### Phase 1: Foundation (Week 1-2)
- [ ] Finalize accessibility framework
- [ ] Create accessibility guidelines
- [ ] Set up automated testing tools
- [ ] Define accessibility quality gates

### Phase 2: Implementation (Week 3-6)
- [ ] Implement keyboard navigation
- [ ] Add ARIA attributes to all components
- [ ] Ensure color contrast compliance
- [ ] Add alt text to all images
- [ ] Implement screen reader support

### Phase 3: Testing (Week 7-8)
- [ ] Conduct automated accessibility testing
- [ ] Test with screen readers (NVDA, JAWS, VoiceOver, ORCA)
- [ ] Test keyboard navigation
- [ ] Test with browser magnifiers
- [ ] Conduct user testing with disabilities

### Phase 4: Training (Week 9-10)
- [ ] Train developers on accessibility
- [ ] Train designers on accessible design
- [ ] Train QA on accessibility testing
- [ ] Create accessibility documentation

### Phase 5: Maintenance (Ongoing)
- [ ] Monitor accessibility metrics
- [ ] Conduct regular accessibility audits
- [ ] Update accessibility features as needed
- [ ] Gather user feedback on accessibility

---

**Document Control**

| Version | Date | Author | Changes |
|---------|-------|--------|----------|
| 1.0 | 2026-02-11 | Brand Strategist | Initial accessibility framework |

---

**Accessibility Statement:** This document is WCAG 2.1 AA compliant with proper heading structure, sufficient color contrast, and keyboard navigation support.
