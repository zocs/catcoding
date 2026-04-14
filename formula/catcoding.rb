class Catcoding < Formula
  desc "Multi-Agent collaborative software development framework with cat themes"
  homepage "https://catcoding.org"
  url "https://github.com/catcoding-dev/catcoding/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/catcoding-dev/catcoding.git", branch: "main"

  depends_on "rust" => :build
  depends_on "nats-server" => :recommended

  def install
    system "cargo", "install", *std_cargo_args(path: "daemon")
    system "cargo", "install", *std_cargo_args(path: "cli")

    # 安装配置示例
    pkgshare.install "config/agent.yaml.example"
    
    # 创建数据目录
    (var/"catcoding").mkpath
  end

  def caveats
    <<~EOS
      🐱 CatCoding 已安装！

      快速开始:
        1. cd your-project
        2. catcoding init
        3. catcoding serve

      Dashboard 地址: http://localhost:8080/dashboard

      配置文件示例: #{pkgshare}/agent.yaml.example

      提示: 如果使用 NATS，请先启动 nats-server
    EOS
  end

  service do
    run [opt_bin/"catcoding-daemon", "serve"]
    keep_alive true
    log_path var/"catcoding/logs/daemon.log"
    error_log_path var/"catcoding/logs/daemon.error.log"
    working_dir var/"catcoding"
  end

  test do
    # 测试版本输出
    assert_match "CatCoding", shell_output("#{bin}/catcoding --version")
    
    # 测试 init 命令
    system "#{bin}/catcoding", "init"
    assert_predicate testpath/".agent.yaml", :exist?
  end
end
