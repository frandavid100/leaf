Leaf 0.1.0-alpha.32

- Update the pinned WordCraft engine from 0.14.0 to the latest stable 0.16.0, including its layout, DOCX preservation, shared Undo data, image-placement and PDF field improvements.
- Recopy the formats crate from that exact revision and retain a smaller, documented ODT compatibility delta for header/footer links, dormant parts, geometry and proofing language metadata.
- Save editable footnotes, comment threads, bookmark ranges, floating pictures and newly authored basic equations as ODT. Preserve real custom column widths and use their effective gap in the native controls.
- Embed the engine's pinned Language data, require it in release builds and ship its licenses, also available from About Leaf.

Client performance and code-organization improvements are being validated against a same-engine control. Remaining ODT fidelity guards are retained for unsupported options; an engine upgrade is not a claim of full format parity. Publication is explicitly authorized after native and installed GNOME 49/51 validation.
