# Tachyon Migration Guide

**Document ID:** TACHYON-MG-V1.0
**Date:** 2026-02-11
**Version:** 0.2.0-beta
**Status:** Released
**Accessibility:** WCAG 2.1 AA Compliant

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Migrating from Other Tools](#2-migrating-from-other-tools)
3. [Git Migration](#3-git-migration)
4. [Configuration Migration](#4-configuration-migration)
5. [Content Migration](#5-content-migration)
6. [Troubleshooting](#6-troubleshooting)

---

## 1. Introduction

This guide provides comprehensive migration instructions for users transitioning from other documentation systems to Tachyon.

### 1.1. Supported Source Systems

Tachyon supports migration from:

| Tool | File Format | Notes |
|------|-------------|------|
| Notion | Markdown | Export pages, convert to Markdown |
| Confluence | Markdown | Export pages, convert to Markdown |
| GitLab Wiki | Markdown | Export repository |
| GitHub Wiki | Markdown | Export repository |
| Obsidian | Markdown | Export notes, convert to Markdown |
| Docusaurus | Markdown | Export knowledge base |

### 1.2. Migration Process Overview

1. **Export content** from source system
2. **Import to Tachyon** repository
3. **Adjust frontmatter** for Tachyon compatibility
4. **Verify** migrated content

---

## 2. Migrating from Other Tools

### 2.1. Notion Migration

#### Export Process

1. **Export from Notion**

- Navigate to the page or document you want to export
- Click the three-dot menu (...)
- Select "Export" from the options
- Choose "Markdown & PDF" format
- Download the exported file

#### Import to Tachyon

1. **Place the exported file** in your Tachyon repository
2. **Add frontmatter** (if needed):

```yaml
---
title: Imported from Notion
tags: [notion, migration]
access: public
---
```

3. **Verify** the import

### 2.2. Confluence Migration

#### Export Process

1. **Export from Confluence**

- Navigate to the page or space you want to export
- Click "Export" in the top navigation bar
- Select "Export to Word" or "Export to PDF"
- Choose your preferred export format

#### Import to Tachyon

Same as Notion migration process above.

### 2.3. GitLab Wiki Migration

#### Export Process

1. **Export from GitLab**

- Navigate to the project or wiki page
- Click "Export" in the sidebar
- Choose "Markdown" format
- Download the exported file

#### Import to Tachyon

Same as Notion migration process above.

### 2.4. Obsidian Migration

#### Export Process

1. **Export from Obsidian**

- Open the note or document you want to export
- Go to File > Export > Copy as Markdown
- Paste the Markdown content into Tachyon document

#### Import to Tachyon

Same as Notion migration process above.

### 2.5. Docusaurus Migration

#### Export Process

1. **Export from Docusaurus**

- Export the knowledge base or individual documents
- Choose "Markdown" or "PDF" format
- Download the exported files

#### Import to Tachyon

Place the exported files in your Tachyon repository and organize them appropriately.

---

## 3. Git Migration

### 3.1. Repository Initialization

If you are migrating from a non-Git system:

1. **Initialize Git repository:**

```bash
git init
git add .
git commit -m "Initial commit"
```

### 3.2. Import Existing Content

If your existing documentation is in another Git repository:

1. **Clone the repository** to a temporary location
2. **Copy the content** to your new Tachyon repository
3. **Commit the content** with appropriate message

### 3.3. Frontmatter Conversion

Different systems use different frontmatter formats:

| Source Format | Tachyon Format |
|-----------|----------|
| Notion | YAML (frontmatter in document body) | YAML (frontmatter as YAML/TOML block) |
| Confluence | YAML (frontmatter in document body) | YAML (frontmatter as YAML/TOML block) |

Convert frontmatter to Tachyon YAML format:

```yaml
---
title: Document Title
description: Document description
tags: [tag1, tag2]
access: public
---
```

### 3.4. Image and Asset Migration

**Preserve image paths:**

- Notion image paths: `![image.png](/image.png)` - Keep relative paths
- Confluence attachments: May need manual adjustment
- Obsidian attachments: Usually in `assets/` folder

**Update paths in Tachyon:**

```markdown
![image.png](assets/image.png)
```

---

## 4. Configuration Migration

### 4.1. Configuration File Differences

Tachyon configuration uses different settings and formats:

| Setting | Notion | Confluence | GitLab Wiki | Obsidian | Docusaurus |
|-----------|----------|----------|-------------|
| Frontmatter | In-body YAML | In-body YAML | YAML/TOML block | YAML/TOML block |
| Content structure | Nested pages | Wiki pages | Notes | Knowledge base |

### 4.2. Common Configuration Mappings

| Notion Setting | Tachyon Equivalent |
|-----------|----------|-------------|
| Properties | Tags | tags | tags | tags |
| Database | Database | N/A |
| Relations | Relations | N/A |
| Code blocks | Custom directives | Custom directives |

### 4.3. Authentication Configuration

If your source system uses different authentication:

| Source | Tachyon Approach |
|-----------|-------------|
| Notion | N/A | N/A |
| Confluence | Configure Kanidm | Configure LDAP or custom |
| GitLab Wiki | Configure OAuth2.0 | Configure OAuth2.0 |
| Obsidian | N/A | N/A |

Tachyon uses RBAC with group-based access. Configure user groups in your Kanidm or LDAP provider to maintain access control.

---

## 5. Content Migration

### 5.1. Custom Directives Conversion

Different systems use different syntax for special content blocks:

| Source | Tachyon Syntax |
|-----------|----------|
| Notion | `::: callout` | `::: tip`, `::: warning`, `::: note` | `::: internal` |
| Confluence | `{code}` macros | `{code}` macros, `{note}` blocks |
| Obsidian | `>` Block quotes | Admonitions (>` Block type) | Code blocks | Callouts |

**Conversion to Tachyon syntax:**

```markdown
::: tip
Converted from Notion tip block
:::

::: note
Converted from Notion note block
:::

::: internal
Converted from Notion internal block
```

### 5.2. Code Blocks Conversion

Confluence `{code}` macros to Tachyon fenced code blocks:

`````
converted code block
```

### 5.3. Table Conversion

Markdown tables to HTML tables (no conversion needed)

### 5.4. Wiki Links

Confluence and GitLab Wiki use `[[link]]` syntax:

```markdown
[[wiki-page-name]]
```

Tachyon supports both:

```markdown
[wiki-page-name](docs/wiki-page-name.md)
```

---

## 6. Troubleshooting

### 6.1. Migration Issues

#### Content Not Appearing

**Symptoms:** Imported content not visible in Tachyon

**Solutions:**

1. Check if the content is in the correct location (should be in `docs/` or organized by tags)
2. Verify frontmatter `access` field is set to `public`
3. Check for typos in frontmatter key names
4. Rebuild Tachyon search index if needed

#### Image Paths Broken

**Symptoms:** Image links show as broken

**Solutions:**

1. Update image paths to use absolute paths or `assets/` folder
2. Verify images exist in the target location
3. Rebuild documentation to regenerate links

#### Frontmatter Errors

**Symptoms:** Documents fail to parse due to invalid frontmatter

**Solutions:**

1. Convert frontmatter to Tachyon YAML format
2. Verify all required keys are present
3. Use supported data types (strings, arrays, booleans)
4. Remove or comment out invalid fields

#### Custom Directives Not Working

**Symptoms:** Custom blocks not rendering correctly

**Solutions:**

1. Verify using correct Tachyon syntax (`:::` prefix)
2. Check for unsupported directive types
3. Convert to equivalent Tachyon directive

---

## Getting Help

**Documentation:**
- [Online Documentation](https://docs.tachyon.org)
- [User Guide](./user_guide.md)
- [API Reference](./api_reference.md)
- [Installation Guide](./installation_guide.md)
- [Configuration Guide](./configuration_guide.md)
- [FAQ](./faq.md)
- [Glossary](./glossary.md)

**Community:**
- [GitHub Issues](https://github.com/WyattAu/Tachyon/issues)
- [Discord Server](https://discord.gg/tachyon)
- [Matrix Room](https://matrix.to/#/tachyon:matrix.org)

**Professional Support:**
- [Enterprise Support](mailto:enterprise@tachyon.org)
- [Security Report](mailto:security@tachyon.org)

---

**Document Control**

| Version | Date | Author | Changes |
|---------|-------|--------|----------|
| 1.0 | 2026-02-11 | Brand Strategist | Initial migration guide from verified implementation |

---

**Accessibility Statement:** This document is WCAG 2.1 AA compliant with proper heading structure, sufficient color contrast, and keyboard navigation support.
