extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // res.set_icon("test.ico");
        // res.set_language(winapi::um::winnt::MAKELANGID(
        //     winapi::um::winnt::LANG_ENGLISH,
        //     winapi::um::winnt::SUBLANG_ENGLISH_US
        // ));
        res.set_manifest(
            r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    </assembly>
    "#,
        );

        res.compile().unwrap();
    }
}
