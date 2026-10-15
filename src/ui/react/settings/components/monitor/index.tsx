import cs from "./index.module.css";

interface Props {
  monitorId: string;
  width?: number;
  height?: number;
}

export function Monitor({ monitorId, width = 1920, height = 1080 }: Props) {
  void monitorId;

  const style: React.CSSProperties = {
    aspectRatio: `${width} / ${height}`,
  };
  if (width > height) {
    style.width = "100%";
  } else {
    style.height = "100%";
  }

  return (
    <div className={cs.monitorContainer}>
      <div className={cs.monitor} style={style}>
        <div className={cs.screen} />
      </div>
    </div>
  );
}
