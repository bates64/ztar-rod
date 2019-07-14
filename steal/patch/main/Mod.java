// 
// Decompiled by Procyon v0.5.36
// Then modified
// 

package main;

import java.io.FileNotFoundException;
import java.io.RandomAccessFile;
import org.apache.commons.io.FileUtils;
import main.config.Options;
import java.io.IOException;
import java.awt.Component;
import shared.SwingUtils;
import main.config.Config;
import java.io.File;

public final class Mod
{
    private static final String CONFIG_FILENAME = "mod.cfg";
    private final File directory;
    public Config config;
    public String name;
    
    public Mod(final File modDirectory) {
        this.directory = modDirectory;
        // Note: changed to use File.separatorChar instead of hard-coded '\\'
        final File modConfig = new File(modDirectory.getAbsolutePath() + File.separatorChar + "mod.cfg");
        try {
            this.readModConfig(modConfig);
        }
        catch (IOException e) {
            SwingUtils.showFramedMessageDialog(null, "IOException while attempting to read mod.cfg", "Config Read Exception", 0);
            System.exit(-1);
        }
    }
    
    private void readModConfig(final File configFile) throws IOException {
        if (!configFile.exists()) {
            final int choice = SwingUtils.showFramedConfirmDialog(null, "Could not find mod config!\nCreate a new one?", "Missing Config", 0, 3);
            if (choice != 0) {
                System.exit(0);
            }
            final boolean success = this.makeNewConfig(configFile);
            if (!success) {
                SwingUtils.showFramedMessageDialog(null, "Failed to create new config.\nPlease try again.", "Create Config Failed", 0);
                System.exit(0);
            }
            this.config.saveConfigFile();
            return;
        }
        (this.config = new Config(configFile, new Options.Scope[] { Options.Scope.Patch, Options.Scope.Editor })).readConfig();
    }
    
    private boolean makeNewConfig(final File configFile) throws IOException {
        FileUtils.touch(configFile);
        this.config = new Config(configFile, new Options.Scope[] { Options.Scope.Patch, Options.Scope.Editor });
        for (final Options opt : Options.values()) {
            switch (opt.scope) {
                case Editor:
                case Patch: {
                    opt.setToDefault(this.config);
                    break;
                }
            }
        }
        return true;
    }
    
    public File getDirectory() {
        return this.directory;
    }
    
    public void prepareNewRom() throws IOException {
        final File targetRom = new File(Directories.MOD_OUT + DevContext.getPristineRomName());
        DevContext.copyPristineRom(targetRom);
    }
    
    public RandomAccessFile getTargetRomWriter() throws FileNotFoundException {
        final File targetRom = new File(Directories.MOD_OUT + DevContext.getPristineRomName());
        return new RandomAccessFile(targetRom, "rw");
    }
    
    public File getTargetRom() {
        return new File(Directories.MOD_OUT + DevContext.getPristineRomName());
    }
}
