#!/bin/bash
# CatCoding 一键安装脚本
# 用法: curl -fsSL https://catcoding.org/install.sh | bash

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 猫咪 ASCII Art
CAT_ART="
    /\\_/\\  
   ( o.o ) 
    > ^ <
   /|   |\\
  (_|   |_)
"

echo -e "${BLUE}${CAT_ART}${NC}"
echo -e "${GREEN}🐱 CatCoding 安装程序${NC}"
echo ""

# 检测操作系统
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

OS=$(detect_os)
echo -e "${BLUE}📦 检测到操作系统: ${OS}${NC}"

# 检查依赖
check_dependency() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}❌ 未找到 $1，请先安装${NC}"
        return 1
    fi
    return 0
}

# 安装 Rust（如果需要）
install_rust() {
    if ! command -v rustc &> /dev/null; then
        echo -e "${YELLOW}📦 正在安装 Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✅ Rust 安装完成${NC}"
    else
        echo -e "${GREEN}✅ Rust 已安装: $(rustc --version)${NC}"
    fi
}

# 下载并安装 CatCoding
install_catcoding() {
    echo -e "${BLUE}📦 正在下载 CatCoding...${NC}"
    
    # 创建安装目录
    INSTALL_DIR="$HOME/.catcoding/bin"
    mkdir -p "$INSTALL_DIR"
    
    # 根据架构选择下载链接
    ARCH=$(uname -m)
    if [[ "$ARCH" == "x86_64" ]]; then
        ARCH="amd64"
    elif [[ "$ARCH" == "aarch64" ]] || [[ "$ARCH" == "arm64" ]]; then
        ARCH="arm64"
    fi
    
    # 下载（暂时使用源码编译，后续会提供预编译版本）
    echo -e "${YELLOW}📦 暂时使用源码编译安装...${NC}"
    
    # 克隆仓库
    TEMP_DIR=$(mktemp -d)
    git clone https://github.com/catcoding-dev/catcoding.git "$TEMP_DIR" 2>/dev/null || {
        echo -e "${YELLOW}⚠️  GitHub 仓库尚未公开，使用本地构建${NC}"
        TEMP_DIR="$HOME/devs/catcoding"
    }
    
    # 编译
    cd "$TEMP_DIR"
    echo -e "${BLUE}🔨 正在编译...${NC}"
    cargo build --release
    
    # 复制二进制文件
    cp target/release/catcoding-daemon "$INSTALL_DIR/"
    cp target/release/catcoding "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/catcoding-daemon"
    chmod +x "$INSTALL_DIR/catcoding"
    
    echo -e "${GREEN}✅ CatCoding 安装完成${NC}"
}

# 配置 PATH
setup_path() {
    INSTALL_DIR="$HOME/.catcoding/bin"
    
    # 检查是否已在 PATH 中
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${BLUE}📝 配置 PATH...${NC}"
        
        # 根据 shell 类型添加
        if [[ -n "$BASH_VERSION" ]]; then
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.bashrc"
            echo -e "${GREEN}✅ 已添加到 ~/.bashrc${NC}"
        elif [[ -n "$ZSH_VERSION" ]]; then
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.zshrc"
            echo -e "${GREEN}✅ 已添加到 ~/.zshrc${NC}"
        else
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.profile"
            echo -e "${GREEN}✅ 已添加到 ~/.profile${NC}"
        fi
        
        # 当前会话生效
        export PATH="$PATH:$INSTALL_DIR"
    fi
}

# 创建示例配置
create_example_config() {
    echo -e "${BLUE}📝 创建示例配置...${NC}"
    
    EXAMPLE_DIR="$HOME/.catcoding/examples"
    mkdir -p "$EXAMPLE_DIR"
    
    # 创建示例 agent.yaml
    cat > "$EXAMPLE_DIR/agent.yaml" << 'EOF'
# CatCoding Agent 配置文件示例
project:
  name: "example-project"
  description: "示例项目"

agents:
  pm:
    enabled: true
    adapter: "hermes"
  core_dev:
    enabled: true
    adapter: "hermes"

watchdog:
  heartbeat_timeout: 30
  max_restarts: 3
EOF
    
    echo -e "${GREEN}✅ 示例配置已创建: $EXAMPLE_DIR/agent.yaml${NC}"
}

# 主安装流程
main() {
    echo -e "${BLUE}🚀 开始安装 CatCoding...${NC}"
    echo ""
    
    # 安装 Rust
    install_rust
    
    # 安装 CatCoding
    install_catcoding
    
    # 配置 PATH
    setup_path
    
    # 创建示例配置
    create_example_config
    
    echo ""
    echo -e "${GREEN}🎉 安装完成！${NC}"
    echo ""
    echo -e "${BLUE}快速开始:${NC}"
    echo -e "  1. 进入你的项目目录"
    echo -e "  2. 运行 ${YELLOW}catcoding init${NC} 初始化"
    echo -e "  3. 运行 ${YELLOW}catcoding serve${NC} 启动 Daemon"
    echo -e "  4. 打开 ${YELLOW}http://localhost:19800/dashboard${NC} 查看 Dashboard"
    echo ""
    echo -e "${BLUE}更多信息:${NC}"
    echo -e "  - 文档: https://catcoding.org/docs"
    echo -e "  - GitHub: https://github.com/catcoding-dev/catcoding"
    echo ""
    echo -e "${GREEN}🐱 让 AI 像猫咪团队一样协作做菜！${NC}"
}

# 运行主函数
main
