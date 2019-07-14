/*
 * Decompiled with CFR 0.145.
 * Then modified.
 */
package main;

import data.battle.ActorTypesEditor;
import data.battle.BattleDumper;
import data.battle.extra.AuxBattleDumper;
import data.battle.struct.Actor;
import data.globals.TableDumper;
import data.globals.WorldMapEditor;
import data.map.MapDumper;
import data.requests.SpecialRequestDumper;
import data.shared.DataConstants;
import data.shared.lib.FunctionLibrary;
import data.sprites.Sprite;
import data.sprites.SpriteDumper;
import data.sprites.editor.SpriteEditor;
import data.strings.StringDumper;
import data.texture.IconDumper;
import data.texture.ImageDumper;
import editor.Editor;
import java.awt.AWTEvent;
import java.awt.Component;
import java.awt.Container;
import java.awt.Desktop;
import java.awt.Dialog;
import java.awt.Dimension;
import java.awt.Image;
import java.awt.LayoutManager;
import java.awt.Toolkit;
import java.awt.Window;
import java.awt.datatransfer.Clipboard;
import java.awt.datatransfer.ClipboardOwner;
import java.awt.datatransfer.StringSelection;
import java.awt.datatransfer.Transferable;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.WindowAdapter;
import java.awt.event.WindowEvent;
import java.awt.event.WindowListener;
import java.io.File;
import java.io.IOException;
import java.io.PrintStream;
import java.io.PrintWriter;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.nio.Buffer;
import java.nio.ByteBuffer;
import java.security.CodeSource;
import java.security.ProtectionDomain;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import javax.swing.AbstractButton;
import javax.swing.ImageIcon;
import javax.swing.JButton;
import javax.swing.JComponent;
import javax.swing.JDialog;
import javax.swing.JFrame;
import javax.swing.JLabel;
import javax.swing.JMenuItem;
import javax.swing.JPanel;
import javax.swing.JPopupMenu;
import javax.swing.JProgressBar;
import javax.swing.JScrollBar;
import javax.swing.JScrollPane;
import javax.swing.JTextArea;
import javax.swing.JTextField;
import javax.swing.SwingUtilities;
import javax.swing.SwingWorker;
import javax.swing.UIManager;
import main.DevContext;
import main.Directories;
import main.InputFileException;
import main.Mod;
import main.config.BuildOptionsPanel;
import main.config.Config;
import main.config.DumpOptionsPanel;
import main.config.Options;
import net.miginfocom.swing.MigLayout;
import org.apache.commons.io.FileUtils;
import patcher.Patcher;
import reports.BattleMapTracker;
import reports.EffectTypeTracker;
import reports.FunctionCallTracker;
import shared.Globals;
import shared.SwingUtils;
import util.IOUtils;
import util.LogFile;
import util.Logger;
import util.Priority;

public class StarRodDev
extends JFrame {
    private static final long serialVersionUID = 1L;
    public static final String STAR_ROD_DEV_VERSION = "0.1.2";
    private static final String TITLE_TEXT = "Star Rod Mod Manager (v0.1.2)";
    private final DumpOptionsPanel dumpOptionsPanel = new DumpOptionsPanel();
    private final BuildOptionsPanel buildOptionsPanel = new BuildOptionsPanel();
    private final JTextArea consoleTextArea;
    private final Logger.Listener consoleListener;
    private final Logger.Listener progressListener;
    private final JPanel progressPanel;
    private final JProgressBar progressBar;
    private final JLabel progressLabel;
    private boolean taskRunning = false;
    private List<JButton> buttons = new ArrayList<JButton>();

    public static void main(String[] args) {
        String jarDir = "";
        try {
            jarDir = new File(StarRodDev.class.getProtectionDomain().getCodeSource().getLocation().toURI()).getParent();
        }
        catch (URISyntaxException e) {
            try {
                e.printStackTrace();
                jarDir = new File(".").getCanonicalPath();
            }
            catch (IOException ioe) {
                System.out.println("Could not find native libraries.\n" + ioe.toString());
                ioe.printStackTrace();
                System.exit(-1);
            }
        }
        // Note: changed to switch library path based on os.name
        if (System.getProperty("os.name") == "Mac OS X") {
            System.setProperty("org.lwjgl.librarypath", jarDir + "/natives/macosx");
        } else {
            System.setProperty("org.lwjgl.librarypath", jarDir + File.separatorChar + "natives");
        }
        try {
            UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName());
        }
        catch (Exception e) {
            Logger.log("Could not set UI to system look and feel.", Priority.ERROR);
        }
        try {
            DevContext.initialize();
            if (!DevContext.mainConfig.getBoolean(Options.Dumped)) {
                new StarRodDev();
            } else {
                GreetingDialog hello = new GreetingDialog();
                GreetingChoice chosenTool = hello.getChoice();
                hello.dispose();
                switch (chosenTool) {
                    case EXIT: {
                        DevContext.exit();
                        break;
                    }
                    case MOD_MANAGER: {
                        new StarRodDev();
                        break;
                    }
                    case MAP_EDITOR: {
                        Editor.launchEditor();
                        break;
                    }
                    case SPRITE_EDITOR: {
                        new SpriteEditor();
                        break;
                    }
                    default: {
                        throw new IllegalStateException("Tool does not exist for " + (Object)((Object)chosenTool));
                    }
                }
            }
        }
        catch (Throwable e) {
            SwingUtilities.invokeLater(() -> {
                e.printStackTrace();
                StarRodDev.displayStackTrace(e);
            });
        }
    }

    private StarRodDev() {
        this.setTitle(TITLE_TEXT);
        this.setDefaultCloseOperation(3);
        this.setIconImage(Globals.getDefaultIconImage());
        this.setMinimumSize(new Dimension(480, 32));
        this.setLocationRelativeTo(null);
        JTextField modFolderField = new JTextField();
        modFolderField.setEditable(false);
        modFolderField.setMinimumSize(new Dimension(64, 24));
        modFolderField.setText(DevContext.currentMod.getDirectory().getAbsolutePath());
        JButton chooseFolderButton = new JButton("Change");
        chooseFolderButton.addActionListener(e -> {
            File choice = DevContext.promptSelectModDirectory();
            if (choice != null) {
                modFolderField.setText(choice.getAbsolutePath());
            }
        });
        this.buttons.add(chooseFolderButton);
        final JTextField romFileField = new JTextField();
        romFileField.setEditable(false);
        romFileField.setMinimumSize(new Dimension(64, 24));
        romFileField.setText(DevContext.getPristineRomPath());
        JButton chooseROMButton = new JButton("Change");
        chooseROMButton.addActionListener(e -> new Thread(){

            @Override
            public void run() {
                for (Object button : StarRodDev.this.buttons) {
                    ((AbstractButton)button).setEnabled(false);
                }
                File choice = DevContext.promptSelectPristineRom();
                if (choice != null) {
                    romFileField.setText(choice.getAbsolutePath());
                }
                for (JButton button : StarRodDev.this.buttons) {
                    button.setEnabled(true);
                }
            }
        }.start());
        this.buttons.add(chooseROMButton);
        JButton dumpROMButton = new JButton("Dump ROM Assets");
        dumpROMButton.addActionListener(e -> {
            int choice = 0;
            Config cfg = DevContext.mainConfig;
            if (cfg.getBoolean(Options.Dumped)) {
                choice = SwingUtils.showFramedConfirmDialog(null, "Exisiting files will be overwritten.\r\nDo you wish to continue?", "Overwrite Warning", 0, 2);
            }
            if (choice == 0) {
                this.startTask(new TaskWorker(() -> {
                    this.dumpAssets();
                    if (cfg.getBoolean(Options.CleanDump)) {
                        cfg.setBoolean(Options.CleanDump, false);
                        cfg.saveConfigFile();
                    }
                    if (!cfg.getBoolean(Options.Dumped)) {
                        cfg.setBoolean(Options.Dumped, true);
                        cfg.saveConfigFile();
                    }
                }));
            }
        });
        this.buttons.add(dumpROMButton);
        JButton dumpOptionsButton = new JButton("Options");
        dumpOptionsButton.addActionListener(e -> {
            Config cfg = DevContext.mainConfig;
            this.dumpOptionsPanel.setValues(cfg);
            int userAction = SwingUtils.showFramedConfirmDialog(this, this.dumpOptionsPanel, "Dump Options", 2);
            if (userAction == 0) {
                this.dumpOptionsPanel.getValues(cfg);
                cfg.saveConfigFile();
            }
        });
        this.buttons.add(dumpOptionsButton);
        JButton createModButton = new JButton("Copy Assets to Mod");
        createModButton.addActionListener(e -> {
            if (!DevContext.mainConfig.getBoolean(Options.Dumped)) {
                SwingUtils.showFramedMessageDialog(null, "You must dump assets before copying them to your mod.", "Dump Required", 2);
                return;
            }
            int choice = SwingUtils.showFramedConfirmDialog(null, "Any exisiting mod files will be overwritten.\r\nDo you wish to continue?", "Copy Assets to Mod", 0, 2);
            if (choice == 0) {
                this.startTask(new TaskWorker(() -> this.copyAssets()));
            }
        });
        this.buttons.add(createModButton);
        JButton compileModButton = new JButton("Compile Mod");
        compileModButton.addActionListener(e -> this.startTask(new TaskWorker(() -> this.compileMod())));
        this.buttons.add(compileModButton);
        JButton compileOptionsButton = new JButton("Options");
        compileOptionsButton.addActionListener(e -> {
            Config cfg = DevContext.currentMod.config;
            this.buildOptionsPanel.setValues(cfg);
            int userAction = SwingUtils.showFramedConfirmDialog(this, this.buildOptionsPanel, "Build Options", 2);
            if (userAction == 0) {
                this.buildOptionsPanel.getValues(cfg);
                cfg.saveConfigFile();
            }
        });
        this.buttons.add(compileOptionsButton);
        JButton packageModButton = new JButton("Package Mod");
        packageModButton.addActionListener(e -> this.startTask(new TaskWorker(() -> this.packageMod())));
        this.buttons.add(packageModButton);
        this.consoleTextArea = new JTextArea();
        this.consoleTextArea.setRows(20);
        this.consoleTextArea.setEditable(false);
        JScrollPane consoleScrollPane = new JScrollPane(this.consoleTextArea);
        consoleScrollPane.setHorizontalScrollBarPolicy(30);
        consoleScrollPane.setVisible(false);
        this.consoleListener = msg -> {
            this.consoleTextArea.append(msg.text + "\r\n");
            JScrollBar vertical = consoleScrollPane.getVerticalScrollBar();
            vertical.setValue(vertical.getMaximum());
        };
        JMenuItem toggleConsole = new JMenuItem("Show Console");
        JPopupMenu popupMenu = new JPopupMenu();
        popupMenu.add(toggleConsole);
        ((JComponent)this.getContentPane()).setComponentPopupMenu(popupMenu);
        toggleConsole.addActionListener(e -> {
            if (consoleScrollPane.isVisible()) {
                toggleConsole.setText("Show Console");
                consoleScrollPane.setVisible(false);
            } else {
                toggleConsole.setText("Hide Console");
                consoleScrollPane.setVisible(true);
            }
            this.revalidate();
            this.pack();
        });
        JMenuItem copyText = new JMenuItem("Copy Text");
        JPopupMenu copyTextMenu = new JPopupMenu();
        copyTextMenu.add(copyText);
        consoleScrollPane.setComponentPopupMenu(copyTextMenu);
        copyText.addActionListener(e -> {
            StringSelection stringSelection = new StringSelection(this.consoleTextArea.getText());
            Clipboard cb = Toolkit.getDefaultToolkit().getSystemClipboard();
            cb.setContents(stringSelection, null);
        });
        this.progressLabel = new JLabel("The march of progress.");
        this.progressBar = new JProgressBar();
        this.progressBar.setIndeterminate(true);
        this.progressPanel = new JPanel();
        this.progressPanel.setLayout(new MigLayout("fillx"));
        this.progressPanel.add((Component)this.progressLabel, "wrap");
        this.progressPanel.add((Component)this.progressBar, "grow, wrap 8");
        this.progressPanel.setVisible(false);
        this.progressListener = msg -> SwingUtilities.invokeLater(() -> this.progressLabel.setText(msg.text));
        this.setDefaultCloseOperation(0);
        this.addWindowListener(new WindowAdapter(){

            @Override
            public void windowClosing(WindowEvent e) {
                int choice = 0;
                if (StarRodDev.this.taskRunning) {
                    choice = SwingUtils.showFramedConfirmDialog(null, "A task is still running.\r\nAre you sure you want to exit?", "Task Still Running", 0, 2);
                }
                if (choice == 0) {
                    System.exit(0);
                }
            }
        });
        this.setLayout(new MigLayout("fillx, ins 16 16 n 16, hidemode 3"));
        this.add(new JLabel("Mod Folder:"));
        this.add((Component)modFolderField, "pushx, growx");
        this.add((Component)chooseFolderButton, "wrap");
        this.add(new JLabel("Target ROM:"));
        this.add((Component)romFileField, "pushx, growx");
        this.add((Component)chooseROMButton, "wrap 16");
        this.add((Component)dumpROMButton, "w 36%, span 3, split 3, center");
        this.add((Component)dumpOptionsButton, "w 8%");
        this.add((Component)createModButton, "w 36%, wrap 8");
        this.add((Component)compileModButton, "w 36%, span 3, split 3, center");
        this.add((Component)compileOptionsButton, "w 8%");
        this.add((Component)packageModButton, "w 36%, wrap 8");
        this.add((Component)this.progressPanel, "grow, span, wrap");
        this.add((Component)consoleScrollPane, "grow, span, wrap 8");
        this.pack();
        this.setResizable(false);
        this.setVisible(true);
        Logger.addListener(this.consoleListener);
    }

    private void startTask(SwingWorker<?, ?> worker) {
        this.taskRunning = true;
        for (JButton button : this.buttons) {
            button.setEnabled(false);
        }
        Logger.setProgressListener(this.progressListener);
        this.consoleTextArea.setText("");
        this.progressLabel.setText("");
        this.progressPanel.setVisible(true);
        this.revalidate();
        this.pack();
        worker.execute();
    }

    private void endTask() {
        for (JButton button : this.buttons) {
            button.setEnabled(true);
        }
        Logger.removeProgressListener();
        this.progressPanel.setVisible(false);
        this.progressLabel.setText("");
        this.revalidate();
        this.pack();
        this.taskRunning = false;
    }

    private void dumpAssets() {
        LogFile dumpLog = null;
        try {
            boolean fullDump;
            DevContext.mainConfig.readConfig();
            boolean clean = DevContext.mainConfig.getBoolean(Options.CleanDump);
            boolean bl = fullDump = !DevContext.mainConfig.getBoolean(Options.Dumped) || clean;
            if (Directories.getDumpPath() == null) {
                throw new IOException("Dump directory is not set.");
            }
            File dumpDirectory = new File(Directories.getDumpPath());
            if (fullDump && dumpDirectory.exists()) {
                FileUtils.cleanDirectory(dumpDirectory);
            }
            Directories.createDumpDirectories();
            dumpLog = new LogFile(new File(dumpDirectory.getAbsolutePath() + "/dump.log"), false);
            Logger.log("Starting ROM dump: " + new Date().toString(), Priority.IMPORTANT);
            Config cfg = DevContext.mainConfig;
            if (cfg.getBoolean(Options.DumpReports)) {
                BattleMapTracker.enable();
            }
            if (fullDump || cfg.getBoolean(Options.DumpStrings)) {
                Logger.log("Dumping strings...", Priority.MILESTONE);
                StringDumper.dumpAllStrings();
            }
            if (fullDump || cfg.getBoolean(Options.DumpTables)) {
                Logger.log("Dumping tables...", Priority.MILESTONE);
                TableDumper.dumpTables();
            }
            FileUtils.copyFile(new File((Object)((Object)Directories.DATABASE) + "SavedBytes.txt"), new File((Object)((Object)Directories.DUMP_GLOBALS) + "ModBytes.txt"));
            FileUtils.copyFile(new File((Object)((Object)Directories.DATABASE) + "SavedFlags.txt"), new File((Object)((Object)Directories.DUMP_GLOBALS) + "ModFlags.txt"));
            FileUtils.touch(new File((Object)((Object)Directories.DUMP_GLOBALS) + "ModItems.txt"));
            if (fullDump || cfg.getBoolean(Options.DumpMaps)) {
                Logger.log("Dumping maps...", Priority.MILESTONE);
                MapDumper.dumpMaps(fullDump);
            }
            ByteBuffer fileBuffer = DevContext.getPristineRomBuffer();
            FunctionCallTracker.clear();
            ActorTypesEditor.dump();
            if (fullDump || cfg.getBoolean(Options.DumpBattles)) {
                Logger.log("Dumping battles...", Priority.MILESTONE);
                BattleDumper.dumpBattles(fileBuffer);
            }
            if (fullDump || cfg.getBoolean(Options.DumpMoves)) {
                Logger.log("Dumping moves...", Priority.MILESTONE);
                AuxBattleDumper.dumpMoves(fileBuffer);
                AuxBattleDumper.dumpPartnerMoves(fileBuffer);
                AuxBattleDumper.dumpStarPowers(fileBuffer);
                AuxBattleDumper.dumpItemScripts(fileBuffer);
            }
            if (cfg.getBoolean(Options.DumpReports)) {
                PrintWriter pw = IOUtils.getBufferedPrintWriter((Object)((Object)Directories.DUMP_REPORTS) + "enemy_names.txt");
                for (int i = 0; i < 212; ++i) {
                    fileBuffer.position(1767908 + 4 * i);
                    int nameStringID = fileBuffer.getInt();
                    fileBuffer.position(1774712 + 4 * i);
                    int tattleStringID = fileBuffer.getInt();
                    String actorName = DataConstants.getActorName(i);
                    String origin = Actor.nameIDs[i] == null ? "unused" : Actor.nameIDs[i];
                    pw.printf("%02X  %08X %08X  %% %-16s (%s)\r\n", i, nameStringID, tattleStringID, actorName, origin);
                }
                pw.close();
                FileUtils.forceMkdir(Directories.DUMP_REQUESTS.toFile());
                SpecialRequestDumper.dumpRequestedScripts();
                SpecialRequestDumper.dumpRequestedFunctions();
                FunctionCallTracker.printCalls(DataConstants.battleFunctionLib, new PrintWriter((Object)((Object)Directories.DUMP_REPORTS) + "battle_func_list.txt"));
                BattleMapTracker.printBattles();
                BattleMapTracker.printMaps();
                EffectTypeTracker.printEffects(new PrintWriter((Object)((Object)Directories.DUMP_REPORTS) + "used_effects.txt"));
            }
            if (fullDump || cfg.getBoolean(Options.DumpTextures)) {
                Logger.log("Dumping textures...", Priority.MILESTONE);
                ImageDumper.dumpTextures();
            }
            if (fullDump || cfg.getBoolean(Options.DumpIcons)) {
                Logger.log("Dumping icons...", Priority.MILESTONE);
                IconDumper.dumpIcons();
            }
            if (fullDump || cfg.getBoolean(Options.DumpSprites)) {
                Logger.log("Dumping sprites...", Priority.MILESTONE);
                SpriteDumper.dumpSprites();
                Logger.log("Converting sprites...", Priority.MILESTONE);
                Sprite.convertAll();
            }
            WorldMapEditor.dump();
            Logger.log("Finished ROM dump: " + new Date().toString(), Priority.IMPORTANT);
            SwingUtilities.invokeLater(() -> {
                this.revalidate();
                this.pack();
                Toolkit.getDefaultToolkit().beep();
                SwingUtils.showFramedMessageDialog(null, "All assets have been dumped.", "Asset Dump Complete", 1);
            });
        }
        catch (Throwable e) {
            SwingUtilities.invokeLater(() -> {
                e.printStackTrace();
                StarRodDev.displayStackTrace(e);
            });
        }
        if (dumpLog != null) {
            dumpLog.close();
        }
    }

    private void copyAssets() {
        try {
            File dumpDirectory = new File(Directories.getDumpPath());
            if (!dumpDirectory.exists()) {
                SwingUtils.showFramedMessageDialog(null, "Could not find dump directory.\nYou must dump assets before copying them to your mod.", "Missing Dump Directory", 2);
                return;
            }
            if (Directories.getModPath() == null) {
                throw new IOException("Mod directory is not set.");
            }
            Directories.createModDirectories();
            Directories.copyIfMissing(Directories.DUMP_MAP, Directories.MOD_MAP, "AssetTable.txt");
            Directories.copyIfMissing(Directories.DUMP_MAP, Directories.MOD_MAP, "MapTable.xml");
            FileUtils.cleanDirectory(Directories.MOD_MAP_SRC.toFile());
            FileUtils.copyDirectory(Directories.DUMP_MAP_SRC.toFile(), Directories.MOD_MAP_SRC.toFile());
            Directories.copyIfMissing(Directories.DUMP_BATTLE, Directories.MOD_BATTLE, "BattleSections.txt");
            Directories.copyIfMissing(Directories.DUMP_BATTLE, Directories.MOD_BATTLE, "ActorTypes.xml");
            FileUtils.cleanDirectory(Directories.MOD_BATTLE_SRC.toFile());
            FileUtils.copyDirectory(Directories.DUMP_BATTLE_SRC.toFile(), Directories.MOD_BATTLE_SRC.toFile());
            FileUtils.cleanDirectory(Directories.MOD_BATTLE_ENEMY.toFile());
            FileUtils.copyDirectory(Directories.DUMP_BATTLE_ENEMY.toFile(), Directories.MOD_BATTLE_ENEMY.toFile());
            Directories.copyIfMissing(Directories.DUMP_MOVE, Directories.MOD_MOVE, "Moves.txt");
            FileUtils.cleanDirectory(Directories.MOD_MOVE_SRC.toFile());
            FileUtils.copyDirectory(Directories.DUMP_MOVE_SRC.toFile(), Directories.MOD_MOVE_SRC.toFile());
            FileUtils.cleanDirectory(Directories.MOD_ALLY_SRC.toFile());
            FileUtils.copyDirectory(Directories.DUMP_ALLY_SRC.toFile(), Directories.MOD_ALLY_SRC.toFile());
            Directories.copyIfMissing(Directories.DUMP_ITEM, Directories.MOD_ITEM, "Items.txt");
            FileUtils.cleanDirectory(Directories.MOD_ITEM_SRC.toFile());
            FileUtils.copyDirectory(Directories.DUMP_ITEM_SRC.toFile(), Directories.MOD_ITEM_SRC.toFile());
            FileUtils.cleanDirectory(Directories.MOD_STARS_SRC.toFile());
            FileUtils.copyDirectory(Directories.DUMP_STARS_SRC.toFile(), Directories.MOD_STARS_SRC.toFile());
            Directories.copyIfEmpty(Directories.DUMP_IMG_TEX, Directories.MOD_IMG_TEX);
            Directories.copyIfEmpty(Directories.DUMP_IMG_BG, Directories.MOD_IMG_BG);
            Directories.copyIfEmpty(Directories.DUMP_IMG_MISC, Directories.MOD_IMG_MISC);
            Directories.copyIfMissing(Directories.DUMP_SPRITE, Directories.MOD_SPRITE, "Sprites.txt");
            Directories.copyIfEmpty(Directories.DUMP_SPRITE_SRC, Directories.MOD_SPRITE_SRC);
            Directories.copyAllMissing(Directories.DUMP_GLOBALS, Directories.MOD_GLOBALS);
            Directories.copyIfEmpty(Directories.DEFAULTS_MAP, Directories.MOD_MAP_PATCH, true);
            Directories.copyIfEmpty(Directories.DEFAULTS_BATTLE, Directories.MOD_BATTLE_PATCH, true);
            Directories.copyIfEmpty(Directories.DEFAULTS_MOVE, Directories.MOD_MOVE_PATCH, true);
            Directories.copyIfEmpty(Directories.DEFAULTS_ALLY, Directories.MOD_ALLY_PATCH, true);
            Directories.copyIfEmpty(Directories.DEFAULTS_ITEM, Directories.MOD_ITEM_PATCH, true);
            Directories.copyIfEmpty(Directories.DEFAULTS_STARS, Directories.MOD_STARS_PATCH, true);
            SwingUtilities.invokeLater(() -> {
                this.revalidate();
                this.pack();
                Toolkit.getDefaultToolkit().beep();
                SwingUtils.showFramedMessageDialog(null, "Ready to begin modding.", "Asset Copy Complete", 1);
            });
        }
        catch (Throwable e) {
            SwingUtilities.invokeLater(() -> {
                e.printStackTrace();
                StarRodDev.displayStackTrace(e);
            });
        }
    }

    private void compileMod() {
        LogFile compileLog = null;
        try {
            if (Directories.getModPath() == null) {
                throw new IOException("Mod directory is not set.");
            }
            compileLog = new LogFile(new File(Directories.getModPath() + "/compile.log"), false);
            DevContext.currentMod.prepareNewRom();
            DevContext.currentMod.config.readConfig();
            new Patcher();
            SwingUtilities.invokeLater(() -> {
                this.revalidate();
                this.pack();
                Toolkit.getDefaultToolkit().beep();
                SwingUtils.showFramedMessageDialog(null, "Finished compiling mod.", "Mod Compiled", 1);
            });
        }
        catch (Throwable e) {
            SwingUtilities.invokeLater(() -> {
                e.printStackTrace();
                StarRodDev.displayStackTrace(e);
            });
        }
        if (compileLog != null) {
            compileLog.close();
        }
    }

    private void packageMod() {
        try {
            DevContext.currentMod.config.readConfig();
            File patchedRom = DevContext.currentMod.getTargetRom();
            if (!patchedRom.exists()) {
                SwingUtils.showFramedMessageDialog(null, "Could not find patched ROM.\nYou must compile your mod before it can be packaged.", "Missing Patched ROM", 2);
                return;
            }
            Patcher.packageMod(patchedRom);
            SwingUtilities.invokeLater(() -> {
                this.revalidate();
                this.pack();
                Toolkit.getDefaultToolkit().beep();
                SwingUtils.showFramedMessageDialog(null, "Mod package is ready.", "Mod Package Ready", 1);
            });
        }
        catch (Throwable e) {
            SwingUtilities.invokeLater(() -> {
                StarRodDev.displayStackTrace(e);
                e.printStackTrace();
            });
        }
    }

    public static void displayStackTrace(Throwable e) {
        StackTraceElement[] stackTrace = e.getStackTrace();
        JTextArea textArea = new JTextArea(20, 50);
        textArea.setEditable(false);
        JScrollPane detailScrollPane = new JScrollPane(textArea);
        detailScrollPane.setHorizontalScrollBarPolicy(30);
        String title = e.getClass().getSimpleName();
        if (title.isEmpty()) {
            title = "Anonymous Exception";
        }
        if (e instanceof AssertionError) {
            title = "Assertion Failed";
        }
        Logger.log(title, Priority.ERROR);
        Logger.log(e.getMessage(), Priority.IMPORTANT);
        textArea.append(e.getClass() + "\r\n");
        for (StackTraceElement ele : stackTrace) {
            Logger.log("  at " + ele, Priority.IMPORTANT);
            textArea.append("  at " + ele + "\r\n");
        }
        StringBuilder msgBuilder = new StringBuilder();
        File inputFile = null;
        if (e instanceof InputFileException) {
            InputFileException ifx = (InputFileException)e;
            if (ifx.hasFile()) {
                inputFile = ifx.getFile();
                msgBuilder.append(inputFile.getName());
                if (ifx.hasLineNumber()) {
                    msgBuilder.append(String.format(" [Line %d]", ifx.getLineNumber()));
                }
            } else {
                msgBuilder.append("Unspecified Source File");
            }
            msgBuilder.append("\r\n");
        }
        if (e.getMessage() != null) {
            msgBuilder.append(e.getMessage());
        } else if (stackTrace.length > 0) {
            msgBuilder.append("at " + stackTrace[0].toString() + "\r\n");
        }
        Object[] options = inputFile == null ? new String[]{"OK", "Details"} : new String[]{"OK", "Details", "Open File"};
        int selection = SwingUtils.showFramedOptionDialog(null, msgBuilder.toString(), title, 1, 0, Globals.ICON_ERROR, options, options[0]);
        if (selection == 1) {
            options = new String[]{"OK", "Copy to Clipboard"};
            selection = SwingUtils.showFramedOptionDialog(null, detailScrollPane, "Exception Details", 1, 0, Globals.ICON_ERROR, options, options[0]);
            if (selection == 1) {
                StringSelection stringSelection = new StringSelection(textArea.getText());
                Clipboard cb = Toolkit.getDefaultToolkit().getSystemClipboard();
                cb.setContents(stringSelection, null);
            }
        }
        if (selection == 2) {
            try {
                Desktop.getDesktop().open(inputFile);
            }
            catch (IOException openDefaultIOE) {
                try {
                    if (Globals.osName.startsWith("Windows")) {
                        Runtime rs = Runtime.getRuntime();
                        rs.exec("notepad " + inputFile.getCanonicalPath());
                    } else {
                        openDefaultIOE.printStackTrace();
                    }
                }
                catch (IOException nativeIOE) {
                    nativeIOE.printStackTrace();
                }
            }
        }
    }

    private static class GreetingDialog
    extends JDialog {
        private static final long serialVersionUID = 1L;
        private GreetingChoice selected = null;

        public GreetingDialog() {
            super((Dialog)null, true);
            this.addWindowListener(new WindowAdapter(){

                @Override
                public void windowClosing(WindowEvent e) {
                    if (selected == null) {
                        selected = GreetingChoice.EXIT;
                    }
                }
            });
            JButton modManagerButton = new JButton("Mod Manager");
            SwingUtils.setFontSize(modManagerButton, 12.0f);
            modManagerButton.addActionListener(e -> {
                this.selected = GreetingChoice.MOD_MANAGER;
                this.dispatchEvent(new WindowEvent(this, 201));
            });
            JButton mapEditorButton = new JButton("Map Editor");
            SwingUtils.setFontSize(mapEditorButton, 12.0f);
            mapEditorButton.addActionListener(e -> {
                this.selected = GreetingChoice.MAP_EDITOR;
                this.dispatchEvent(new WindowEvent(this, 201));
            });
            JButton spriteEditorButton = new JButton("Sprite Editor");
            SwingUtils.setFontSize(spriteEditorButton, 12.0f);
            spriteEditorButton.addActionListener(e -> {
                this.selected = GreetingChoice.SPRITE_EDITOR;
                this.dispatchEvent(new WindowEvent(this, 201));
            });
            this.setTitle("Star Rod Mod Suite (v1.0)");
            this.setIconImage(Globals.getDefaultIconImage());
            this.setMinimumSize(new Dimension(240, 160));
            this.setLocationRelativeTo(null);
            this.setLayout(new MigLayout("fill"));
            this.add((Component)modManagerButton, "grow, sg buttons, wrap");
            this.add((Component)mapEditorButton, "grow, sg buttons, wrap");
            this.add((Component)spriteEditorButton, "grow, sg buttons");
            this.pack();
            this.setResizable(false);
            this.setVisible(true);
        }

        public GreetingChoice getChoice() {
            return this.selected;
        }

    }

    private static enum GreetingChoice {
        EXIT,
        MOD_MANAGER,
        MAP_EDITOR,
        SPRITE_EDITOR;
        
    }

    private class TaskWorker
    extends SwingWorker<Boolean, String> {
        private final Runnable runnable;

        private TaskWorker(Runnable runnable) {
            this.runnable = runnable;
        }

        @Override
        protected Boolean doInBackground() {
            this.runnable.run();
            return true;
        }

        @Override
        protected void done() {
            StarRodDev.this.endTask();
        }
    }

}

