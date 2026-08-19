use super::*;

#[test]
fn every_resource_has_a_short_unique_freight_label() {
    let mut labels = std::collections::HashSet::new();
    for resource in ResourceType::ALL {
        let label = freight_label(resource);
        assert!(label.len() <= 5);
        assert!(labels.insert(label));
    }
}
