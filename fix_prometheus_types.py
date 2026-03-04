import re

file_path = 'layanan/secreton/crates/replication/src/metrics.rs'

with open(file_path, 'r') as f:
    content = f.read()

content = content.replace('.with_label_values(&[])', '.with_label_values::<&str>(&[])')
content = content.replace('.with_label_values(&[&self.node_id, operation_type, status])', '.with_label_values(&[self.node_id.as_str(), operation_type, status])')
content = content.replace('.with_label_values(&[&self.node_id, operation_type])', '.with_label_values(&[self.node_id.as_str(), operation_type])')

with open(file_path, 'w') as f:
    f.write(content)
