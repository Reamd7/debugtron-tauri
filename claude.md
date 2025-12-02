# Debugtron Tauri 项目文档索引

> 主索引文档 - 引用所有专题文档

**📖 本文件是精简版索引文档，详细内容已拆分为专题文档。**

## 项目概述

**目标**: 将原 Electron 项目迁移到 Tauri 技术栈，使用 Rust 重新实现后端逻辑，前端代码尽量复用。

**原项目**: Debugtron - Debug in-production Electron based App

**核心功能**:
- 自动发现本地 Electron 应用（macOS + Windows）
- 一键启动调试会话（支持 --inspect 和 --inspect-brk 模式）
- 双模式 DevTools 集成（Tauri 窗口模式 + 浏览器模式）
- 实时日志监控和调试目标发现

## 当前状态 🎉

### ✅ Phase 1-8 已完成

**Windows 和 macOS 平台支持已实现**，主要功能包括：

- ✅ **跨平台 Electron 应用发现**（macOS + Windows）
- ✅ **一键调试会话启动**（支持 --inspect 和 --inspect-brk 模式）
- ✅ **双模式 DevTools 集成**（Tauri 窗口模式 + 浏览器模式）
- ✅ **实时日志监控**（Xterm 增量显示优化）
- ✅ **多会话并发调试**（Tab 同步和 SessionId 一致性修复）
- ✅ **Windows PE 资源图标提取**（三级策略 + Windows API）
- ✅ **多平台本地发版系统**（Draft Release 策略 + GitHub CLI）

### 🚧 待完成功能

详见 [TODO.md](TODO.md) 文档。

## 📚 文档索引

### 核心文档

1. **[README.md](README.md)** - 项目主入口文档
   - 项目简介和快速开始
   - 当前功能状态和用户工作流
   - 基本使用指南

2. **[ARCHITECTURE.md](ARCHITECTURE.md)** - 系统架构设计
   - 技术栈和核心模块设计
   - 数据流和关键设计决策
   - 目录结构说明

3. **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - 开发指南
   - 开发环境设置
   - 常用命令和 Git 工作流
   - 代码审查清单和提交规范

4. **[DECISION_LOG.md](DECISION_LOG.md)** - 技术决策记录
   - 详细的技术决策和实现细节
   - 问题分析和解决方案
   - 代码示例和关键实现

5. **[RELEASE_PROCESS.md](RELEASE_PROCESS.md)** - 发版流程指南
   - 多平台本地构建和发布流程
   - Draft Release 策略和脚本详解
   - 故障排查和最佳实践

6. **[TODO.md](TODO.md)** - 待完成事项和路线图
   - 项目待完成功能规划
   - 问题追踪和解决方案
   - 开发路线图和技术债务

### 历史文档

7. **[CLAUDE-full.md](CLAUDE-full.md)**（原完整版） - 完整的历史记录文档
   - 包含所有详细实现记录和里程碑
   - 保留作为历史参考（约1500行）
   - **建议查阅专题文档而非此完整版**

8. **[migration_plan.md](migration_plan.md)** - 迁移计划文档
   - 原项目详细分析
   - Electron → Tauri 架构映射
   - 分阶段实施计划

## 技术栈

### 前端（复用）
- React 19 + TypeScript
- Redux Toolkit + Radix UI + TailwindCSS
- xterm.js（终端显示）

### 后端（Rust 重新实现）
- **框架**: Tauri 1.5
- **核心 Crates**: tokio, serde, anyhow, portpicker, axum, opener
- **平台特定**: macOS (plist), Windows (winapi, image, walkdir)

## 快速开始

```bash
# 开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build

# 运行测试
cargo test
npm test
```

详细开发环境设置见 [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)。

## 发版流程

```bash
# 1. 创建版本和 tag
npm run release 1.0.0

# 2. 在各平台构建上传
# macOS: ./scripts/build-and-upload.sh 1.0.0
# Windows: .\scripts\build-and-upload.ps1 1.0.0

# 3. 发布
npm run publish-release 1.0.0
```

详细流程见 [RELEASE_PROCESS.md](RELEASE_PROCESS.md)。

## 项目维护

### 当前版本
- **版本**: v1.0.3（最新发布版本）
- **状态**: 🎉 **主要功能已完成，待添加 Linux 平台支持**
- **最后更新**: 2025-12-02

### 维护者
- Debugtron Tauri 团队
- Claude Code（开发助手）

---

**💡 提示**: 本文件为精简索引文档，详细内容请查阅对应的专题文档。

**📁 完整历史记录**: 如需查看详细的历史实现记录，请参考原始 CLAUDE-full.md 文件（约1500行）。