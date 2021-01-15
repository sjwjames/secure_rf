1. After you have created the image based on our submission, run the image in three separated docker containers. We name three nodes/containers as P0,P1,TI in the following instructions.
   You should expose at least one port for P0&P1 and expose two ports TI.
   e.g. For TI, execute "docker run -d -p 4000:4000 -p 8000:8000 idash2019:geneX"
        For P0&P1,execute "docker run -d -p 5000:5000 idash2019:geneX"
2. Get the IP address of each container by "docker inspect CONTAINER_ID" for further configuration
2. After three parties are launched in the containers, execute "docker exec -it CONTAINER_ID /bin/bash" for each of them
3. Navigate to the main folder under /usr/src/idash2019-rust in each container
4. Build the program by "cargo build --release", the output executable binary is under target/release/, named as idash2019_rust
5. In container P0, modify the file settings/Party0.toml by changing the ip addresses and ports of P0,P1 and TI in the network section.
   In container P1, modify the file settings/Party1.toml by changing the ip addresses and ports of P0,P1 and TI in the network section.
   In container TI, modify the file settings/TI.toml by changing changing the ip address and ports of TI in the network section.
6. Start TI first in the TI container by "./target/release/idash2019_rust settings/TI.toml"
   Start P0 in the P0 container by "./target/release/idash2019_rust settings/Party0.toml". If you see messages like "client:connection refused by 172.17.0.3:5000", ignore it now, it will disappear after you start P1
   Start P1 in the P0 container by "./target/release/idash2019_rust settings/Party1.toml".
7. By default, it will train a model based on the first fold of BC-TCGA dataset.
    - If you want to test the accuracy of the model, execute "python3 python/accuracy_test.py FOLD" in either P0 or P1 container.
8. When you want to train a model against another fold of the same dataset, you just need to change the fold number under corresponding section.
   e.g. Train against BC-TCGA dataset, changing from fold 1 to fold 2, change settings/Party0.toml in P0 container and settings/Party1.toml in P1 container as follows,

   settings/Party0.toml in P0 container
   x_input_path = "inputs/BC_TCGA_5_folds/1X_train_share0.csv"            x_input_path = "inputs/BC_TCGA_5_folds/2X_train_share0.csv"
   y_input_path = "inputs/BC_TCGA_5_folds/1y_train_share0.csv"    =====>  y_input_path = "inputs/BC_TCGA_5_folds/2y_train_share0.csv"
   output_path = "weights/BC_TCGA_5_folds/fold_1_weights.csv"             output_path = "weights/BC_TCGA_5_folds/fold_2_weights.csv"

   settings/Party1.toml in P1 container
   x_input_path = "inputs/BC_TCGA_5_folds/1X_train_share0.csv"            x_input_path = "inputs/BC_TCGA_5_folds/2X_train_share0.csv"
   y_input_path = "inputs/BC_TCGA_5_folds/1y_train_share0.csv"    =====>  y_input_path = "inputs/BC_TCGA_5_folds/2y_train_share0.csv"
   output_path = "weights/BC_TCGA_5_folds/fold_1_weights.csv"             output_path = "weights/BC_TCGA_5_folds/fold_2_weights.csv"

9. When you want to train a model against another dataset, you need to comment current configuration and comment out the configuration for the other one dataset.
   e.g. From BC-TCGA to GSE2034, comment all the configurations in the BC-TCGA section and comment out the configurations in the GSE2034 section for settings/Party0.toml in P0 container,
   settings/Party1.toml in P1 container and settings/TI.toml in TI container.
   Also do the same thing to python/accuracy_test.py in either P0 or P1 container


For more detailed guidance and reference, check Guidance.txt