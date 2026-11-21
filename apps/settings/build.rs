const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <assemblyIdentity type="win32" name="Whirlpool.Settings" version="1.0.0.0" processorArchitecture="*"/>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <asmv3:application>
    <asmv3:windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </asmv3:windowsSettings>
  </asmv3:application>
</assembly>
"#;

fn main() {
    println!("cargo:rerun-if-changed=../../assets/brand/whirlpool.ico");
    println!("cargo:rerun-if-changed=build.rs");
    let mut resources = winresource::WindowsResource::new();
    resources.set_icon("../../assets/brand/whirlpool.ico");
    resources.set_manifest(MANIFEST);
    resources.set("ProductName", "Whirlpool");
    resources.set("CompanyName", "Whirlpool");
    resources.set(
        "LegalCopyright",
        "Copyright (c) Whirlpool contributors. MIT License.",
    );
    resources.set("OriginalFilename", "whirlpool-settings.exe");
    resources.set("InternalName", "whirlpool-settings");
    resources.set(
        "Comments",
        "Open source: https://github.com/prashantchataut/whirlpool",
    );
    resources.set("FileDescription", "Whirlpool settings");
    if let Err(error) = resources.compile() {
        // Cosmetic only: a missing resource compiler must not block the build.
        println!("cargo:warning=could not embed resources: {error}");
    }
}
