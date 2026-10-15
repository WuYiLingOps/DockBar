import { Select } from "antd";

interface Props {
  value?: string;
  defaultValue?: string;
  onChange?: (value: string) => void;
  style?: React.CSSProperties;
}

// The fonts backend module was removed; offer the common Windows UI font stacks
// so theme font pickers keep working.
const FONT_OPTIONS = [
  "Segoe UI",
  "Segoe UI Variable",
  "Arial",
  "Verdana",
  "Tahoma",
  "Times New Roman",
  "Georgia",
  "Consolas",
  "Courier New",
  "Microsoft YaHei UI",
  "Microsoft JhengHei UI",
  "Yu Gothic UI",
  "Malgun Gothic",
];

export function FontSelect({ value, defaultValue, onChange, style }: Props) {
  const options = FONT_OPTIONS.map((family) => ({
    value: family,
    label: <span style={{ fontFamily: family }}>{family}</span>,
  }));

  return (
    <Select
      showSearch={{ optionFilterProp: "value" }}
      allowClear
      value={value}
      defaultValue={defaultValue}
      onChange={onChange}
      options={options}
      style={{ minWidth: 200, ...style }}
    />
  );
}
