区分“主程序”与“库”
在你规划的这些工程中，大部分是 Library (库)，只有少数几个是 Binary (可执行程序)。

1. 可执行程序 (Binaries) —— 最终跑起来的进程
这些工程目录下会有 src/main.rs 文件。

cmx-server (主程序)
角色：这是你的 ERP 后端主进程。
运行方式：cargo run -p cmx-server。
部署：编译后生成的二进制文件（如 cloud-matrix-server）部署到服务器上。它启动后会监听 HTTP 端口，加载 Wasm 运行时，连接数据库。
cmx-cli (开发工具)
角色：这是给开发者或管理员用的 命令行工具。
运行方式：cmx plugin new my-plugin 或 cmx deploy。
部署：分发给你的插件开发者安装在他们的电脑上。
2. 依赖库 (Libraries) —— 被主程序引用的代码块
这些工程目录下只有 src/lib.rs，没有 main.rs。它们不能单独运行，只能被 cmx-server 或其他库引用。

cmx-core：(Lib) 纯逻辑定义。
cmx-runtime：(Lib) Wasm 引擎逻辑。虽然它很复杂，但它只是被 server 调用的一个模块。
cmx-infra：(Lib) 数据库和基础设施代码。
cmx-protocol：(Lib) 协议定义。
cmx-utils：(Lib) 工具函数。
cmx-pdk：(Lib) 特殊库。它不是给 cmx-server 用的，而是给 插件 用的 SDK。
3. 特殊存在：插件 (Plugins)
plugin-demo / plugin-base
类型：它们通常配置为 cdylib (C-compatible Dynamic Library) 或 bin。
编译目标：它们 不 编译成宿主机的机器码（x86/ARM），而是编译成 .wasm 文件。
运行方式：它们不能直接运行，必须被 cmx-server 加载后才能运行。

依赖调用关系图
理解这个流向很重要：
[可执行文件: cmx-server]  <--- 启动入口  
      |  
      +--- 引用 ---> [库: cmx-infra] (连数据库)  
      +--- 引用 ---> [库: cmx-runtime] (启动虚拟机)  
      |                 |  
      |                 +--- 加载 ---> [文件: plugin-a.wasm]  
      |                                     |  
      |                                     +--- 引用 ---> [库: cmx-pdk] (SDK)  
      |                                     +--- 引用 ---> [库: cmx-protocol]  
      |  
      +--- 引用 ---> [库: cmx-core] (通用类型)  

总结建议
当你写代码时，大部分逻辑（比如加载插件的具体步骤、数据库查询的封装）都写在 Library 里。
cmx-server 应该尽量写得很薄（Thin），它的工作主要是把各个 Library 组装起来，启动 Web 服务。这样做方便后续写单元测试。