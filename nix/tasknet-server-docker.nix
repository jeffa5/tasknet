{
  dockerTools,
  tasknet-server,
}:
dockerTools.buildImage {
  name = "tasknet-server";
  copyToRoot = with dockerTools; [binSh caCertificates];
  config = {
    Cmd = ["${tasknet-server}/bin/server" "--config-file" ./../config.yaml.template];
  };
}
